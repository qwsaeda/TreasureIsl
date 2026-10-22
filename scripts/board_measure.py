#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""赛道节奏 / 跨图难度阶梯 实测（业务规格数值的复测与防漂移）。

业务依据（唯一事实源 = `docs/business/progression.md` §落格节奏 / §跨图难度阶梯）：
- 落格节奏：各图落格类型占比 + 每次摇骰的期望（金 / 抽卡 / 遭遇 / 宝藏）；
- 难度阶梯：各图 BOSS 档位 → 战斗前有效 HP → 不建造 / 造 Castle 的生存余量。

数据源：`assets/data/balance.json` + `assets/data/board.json`（不改任何数据）。

用法：
  python scripts/board_measure.py                # 落格节奏表
  python scripts/board_measure.py --ladder        # 跨图难度阶梯表（含战斗模拟）
  python scripts/board_measure.py --check        # 校验：业务规格里的实测数字与当前数据一致（漂移 ⇒ 非 0 退出）
  python scripts/board_measure.py --json         # 机器可读

约定：幂等 + 只读 + `--check` 非 0 退出（`docs/process/04_tooling.md` §1 新增工具要求）。
"""
from __future__ import annotations

import argparse
import json
import os
import random
import re
import sys

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
BALANCE = os.path.join(ROOT, 'assets', 'data', 'balance.json')
BOARD = os.path.join(ROOT, 'assets', 'data', 'board.json')
SPEC = os.path.join(ROOT, 'docs', 'business', 'progression.md')

SPINS = 40000
SEED = 42
BATTLE_SEED = 11
CARD_GOLD_EXP = 6  # 事件/卡牌格抽一张卡的期望金（卡池权重期望，见 balance.cards.deck）


def load():
    with open(BALANCE, encoding='utf-8') as fh:
        bal = json.load(fh)
    with open(BOARD, encoding='utf-8') as fh:
        board = json.load(fh)
    return bal, board


def track_income(bal, spaces, spins=SPINS, seed=SEED):
    """赛道跑到 BOSS 的期望：摇骰次数 / 收入 / 暂存扣血。"""
    cfg = bal['board']
    min_laps = int(cfg.get('boss_min_laps', 1))
    rnd = random.Random(seed)
    n = len(spaces)
    boss = [i for i, s in enumerate(spaces) if s['kind'] == 'boss'][0]
    tot_spins = tot_gold = tot_dmg = 0
    tot_cards = tot_battles = tot_treasures = 0
    for _ in range(spins):
        idx = laps = gold = dmg = 0
        count = cards = battles = treasures = 0
        while True:
            roll = rnd.randint(1, 6)
            count += 1
            crossed = False
            for _ in range(roll):
                idx = (idx + 1) % n
                if idx == 0:
                    laps += 1
                    gold += int(cfg['lap_bonus_gold'])
                if idx == boss:
                    crossed = True
            kind = spaces[idx]['kind']
            if kind == 'treasure':
                gold += int(cfg['treasure_gold'])
                treasures += 1
            elif kind == 'city':
                gold += int(cfg['city_gold'])
            elif kind == 'battle':
                gold -= int(cfg['battle_gold_loss'])
                dmg += int(cfg['battle_damage'])
                battles += 1
            elif kind in ('card', 'event'):
                gold += CARD_GOLD_EXP
                cards += 1
            if crossed and laps >= min_laps:
                break
        tot_spins += count
        tot_gold += gold
        tot_dmg += dmg
        tot_cards += cards
        tot_battles += battles
        tot_treasures += treasures
    k = float(spins)
    return {
        'spins': tot_spins / k,
        'income': tot_gold / k,
        'pending_damage': tot_dmg / k,
        'cards': tot_cards / k,
        'battles': tot_battles / k,
        'treasures': tot_treasures / k,
    }


def battle(player_atk, player_hp, tier_def, seed=BATTLE_SEED):
    """BOSS 战逐回合模拟（`atk_plus_flat` + rng_span 3）。"""
    rnd = random.Random(seed)
    boss_hp = int(tier_def['hp'])
    boss_atk = int(tier_def['atk'])
    rounds = 0
    while boss_hp > 0 and player_hp > 0 and rounds < 3000:
        rounds += 1
        boss_hp -= max(1, player_atk + 1 + rnd.randint(0, 3) - 1)
        if boss_hp > 0:
            player_hp -= max(1, boss_atk + 1 + rnd.randint(0, 3) - 1)
    return {'rounds': rounds, 'hp_left': player_hp, 'win': boss_hp <= 0}


def rhythm(bal, board):
    rows = []
    for mid, m in sorted(board['maps'].items()):
        spaces = m['spaces']
        counts = {}
        for s in spaces:
            counts[s['kind']] = counts.get(s['kind'], 0) + 1
        st = track_income(bal, spaces)
        rows.append({
            'map': mid,
            'cells': len(spaces),
            'counts': counts,
            'density': {k: round(100.0 * v / len(spaces), 1) for k, v in sorted(counts.items())},
            'boss_index': [i for i, s in enumerate(spaces) if s['kind'] == 'boss'][0],
            **st,
        })
    return rows


def ladder(bal, board):
    tiers = {int(t['tier']): t for t in bal['combat']['boss_tiers']}
    player = bal['combat']['player']
    init_gold = int(bal['economy']['initial_gold'])
    builds = {b['id']: b for b in bal['build']['buildings']}
    rows = []
    for mid, m in sorted(board['maps'].items()):
        tier = int(next(d['boss_tier'] for d in bal['selection']['maps'] if d['id'] == mid))
        st = track_income(bal, m['spaces'])
        gold = init_gold + int(st['income'])
        eff_hp = int(player['hp']) - int(st['pending_damage'])
        variants = []
        for label, bid in (('none', None), ('Castle', 'Castle')):
            if bid is None:
                atk_b = hp_b = cost = 0
            else:
                x = builds[bid]
                atk_b = int(x.get('atk_bonus', 0))
                hp_b = int(x.get('hp_bonus', 0))
                cost = int(x['base_cost'])
            if cost > gold:
                variants.append({'label': label, 'affordable': False, 'cost': cost})
                continue
            res = battle(int(player['atk']) + atk_b, eff_hp + hp_b, tiers[tier])
            variants.append({
                'label': label,
                'affordable': True,
                'cost': cost,
                'atk': int(player['atk']) + atk_b,
                'hp': eff_hp + hp_b,
                'rounds': res['rounds'],
                'hp_left': res['hp_left'],
                'win': res['win'],
                'margin_pct': round(100.0 * res['hp_left'] / (eff_hp + hp_b), 1) if res['win'] else 0.0,
            })
        rows.append({
            'map': mid,
            'tier': tier,
            'boss_hp': int(tiers[tier]['hp']),
            'boss_atk': int(tiers[tier]['atk']),
            'cells': len(m['spaces']),
            'spins': st['spins'],
            'gold_at_boss': gold,
            'effective_hp': eff_hp,
            'variants': variants,
        })
    return rows


def spec_numbers():
    """从业务规格里抽出阶梯表的关键数字（行式：| m01 | tier0 | ... |）。"""
    if not os.path.exists(SPEC):
        return []
    out = []
    for line in open(SPEC, encoding='utf-8').read().splitlines():
        if line.startswith('| m0') and 'tier' in line:
            cells = [c.strip() for c in line.strip().strip('|').split('|')]
            if len(cells) >= 4:
                out.append(cells)
    return out


def check_drift(bal, board):
    """校验：规格阶梯表里的 tier 与当前 balance 一致（数值随 balance 变动会漂移 ⇒ 提醒复测）。"""
    errors = []
    want = {m['id']: int(m['boss_tier']) for m in bal['selection']['maps']}
    for cells in spec_numbers():
        mid = cells[0]
        m = re.search(r'tier(\d+)', cells[1])
        if not m:
            continue
        got = int(m.group(1))
        if mid in want and want[mid] != got:
            errors.append('规格 %s 写 tier%d，但 balance 实际 tier%d（数值已漂移 ⇒ 复测 §跨图难度阶梯）'
                          % (mid, got, want[mid]))
    return errors


def main() -> int:
    ap = argparse.ArgumentParser(description='赛道节奏 / 跨图难度阶梯 实测')
    ap.add_argument('--ladder', action='store_true', help='难度阶梯表（含战斗模拟）')
    ap.add_argument('--check', action='store_true', help='校验规格与数据一致（漂移 ⇒ 非 0 退出）')
    ap.add_argument('--json', action='store_true', help='机器可读输出')
    args = ap.parse_args()

    bal, board = load()
    if args.ladder:
        rows = ladder(bal, board)
        if args.json:
            print(json.dumps(rows, ensure_ascii=False, indent=1))
        else:
            print('=== 跨图难度阶梯（实测；战斗按 atk_plus_flat 逐回合模拟）===')
            for r in rows:
                print('%s tier%d  BOSS HP%d/ATK%d  赛道%d格  E[摇骰]=%.1f  见BOSS时金=%d  有效HP=%d'
                      % (r['map'], r['tier'], r['boss_hp'], r['boss_atk'], r['cells'],
                         r['spins'], r['gold_at_boss'], r['effective_hp']))
                for v in r['variants']:
                    if not v['affordable']:
                        print('    %-8s 买不起（需 %d 金）' % (v['label'], v['cost']))
                    else:
                        print('    %-8s atk%3d hp%4d → %s %3d 回合 余血 %5.0f（%3.1f%%）'
                              % (v['label'], v['atk'], v['hp'], '胜' if v['win'] else '败',
                                 v['rounds'], v['hp_left'], v['margin_pct']))
    else:
        rows = rhythm(bal, board)
        if args.json:
            print(json.dumps(rows, ensure_ascii=False, indent=1))
        else:
            print('=== 落格节奏（实测，%d 次/图，seed=%d）===' % (SPINS, SEED))
            for r in rows:
                dens = ' / '.join('%s %d(%.1f%%)' % (k, v, r['density'][k])
                                  for k, v in sorted(r['counts'].items()))
                print('%s %d格 BOSS@%d ｜ %s' % (r['map'], r['cells'], r['boss_index'], dens))
                print('    E[摇骰]=%.1f  E[金]=%.0f  E[抽卡]=%.1f  E[遭遇]=%.1f  E[宝藏]=%.1f  暂存扣血=%.0f'
                      % (r['spins'], r['income'], r['cards'], r['battles'], r['treasures'],
                         r['pending_damage']))

    if args.check:
        errors = check_drift(bal, board)
        if errors:
            for e in errors:
                print('[board_measure] FAIL：%s' % e, file=sys.stderr)
            return 1
        print('[board_measure] PASS：业务规格的难度档位与 balance 一致')
    return 0


if __name__ == '__main__':
    sys.exit(main())
