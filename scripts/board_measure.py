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
    """赛道跑到 BOSS 的期望：摇骰次数 / 收入 / 暂存扣血 + 领地通道（见 BOSS 累计金）。

    **见 BOSS 累计金（2026-11-19 M6 扩口径）**：旧 `income` 只算「落格净金」
    （宝藏/城镇/绕圈 + 抽卡期望金 − 遭遇失金），不含领地系统（S28 后新增通道）。
    本函数补全**领地通道**（首占奖励 + 每圈领地产出 + 每圈过路费）⇒ `cum_gold`
    = 见 BOSS 时累计金币（落格净金 + 领地通道金）。`progression.md` §跨图节奏实测 的
    「E[见 BOSS 时金]」列 = `initial_gold + cum_gold`（口径 = 含领地通道；漂即回写）。
    """
    cfg = bal['board']
    terr = bal.get('territory', {})
    protos = {p['id']: p for p in terr.get('prototypes', [])}
    prod_coef = float(terr.get('production_per_level', 0.5))
    toll_coef = float(terr.get('toll_per_level', 0.25))
    cap_bonus = int(terr.get('first_capture_bonus', 0))
    min_laps = int(cfg.get('boss_min_laps', 1))
    rnd = random.Random(seed)
    n = len(spaces)
    boss = [i for i, s in enumerate(spaces) if s['kind'] == 'boss'][0]

    # 领地格「每圈产出/过路费」基准（level 1 口径，公式 = base × (1 + coef×level)，
    # 与 `territory.rs::formula` 一致；boss_fort 占格无产出通道，跳过）。
    lap_prod = lap_toll = 0
    for s in spaces:
        if s['kind'] != 'territory' or s.get('isle') == 'boss_fort':
            continue
        p = protos.get(s.get('isle'))
        if p is None:
            continue
        lvl = 1
        lap_prod += int(round(p['production_base'] * (1.0 + prod_coef * lvl)))
        lap_toll += int(round(p['toll_base'] * (1.0 + toll_coef * lvl)))

    tot_spins = tot_gold = tot_dmg = 0
    tot_cum = 0
    tot_cards = tot_battles = tot_treasures = 0
    for _ in range(spins):
        idx = laps = gold = dmg = 0
        count = cards = battles = treasures = 0
        cap_bonus_got = False
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
                # v4 休眠分支（v5 无遭遇格，balance.board 已删 battle_* 键 ⇒ 缺省 0，
                # 此分支 v5 不可达；保留作 v4 地图包兼容面，2026-11-19 登记）。
                battles += 1
            elif kind in ('card', 'event'):
                gold += CARD_GOLD_EXP
                cards += 1
            elif kind == 'territory' and not cap_bonus_got:
                # 落格领地（v5 三支分叉之一 = 副本占领）⇒ 首占奖励（只计一次，
                # 后续同岛两格已归所有，不再首占；产出/过路费走每圈结算通道）。
                gold += cap_bonus
                cap_bonus_got = True
            if crossed and laps >= min_laps:
                break
        # 领地通道：每圈领地产出 + 每圈过路费（首圈绕岛满 min_laps 后，
        # 已占领地在后续 min_laps 圈内持续结算 = 见 BOSS 前累计 min_laps 圈）。
        cum = gold + laps * (lap_prod + lap_toll)
        tot_spins += count
        tot_gold += gold
        tot_cum += cum
        tot_dmg += dmg
        tot_cards += cards
        tot_battles += battles
        tot_treasures += treasures
    k = float(spins)
    return {
        'spins': tot_spins / k,
        'income': tot_gold / k,
        'cum_gold': tot_cum / k,
        'lap_production': lap_prod,
        'lap_toll': lap_toll,
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
            init_gold = int(bal.get('economy', {}).get('initial_gold', 0))
            print('=== 落格节奏（实测，%d 次/图，seed=%d）===' % (SPINS, SEED))
            for r in rows:
                dens = ' / '.join('%s %d(%.1f%%)' % (k, v, r['density'][k])
                                  for k, v in sorted(r['counts'].items()))
                print('%s %d格 BOSS@%d ｜ %s' % (r['map'], r['cells'], r['boss_index'], dens))
                print('    E[摇骰]=%.1f  E[落格净金]=%.0f  E[抽卡]=%.1f  E[遭遇]=%.1f  E[宝藏]=%.1f  暂存扣血=%.0f'
                      % (r['spins'], r['income'], r['cards'], r['battles'], r['treasures'],
                         r['pending_damage']))
                print('    E[见BOSS累计金]=%.0f（落格净金 + 领地通道〔首占 %d + 每圈产出 %d + 每圈过路费 %d〕；'
                      '含 initial_gold %d = %.0f；M6 口径，回填 progression.md §跨图节奏实测）'
                      % (r['cum_gold'], int(bal.get('territory', {}).get('first_capture_bonus', 0)),
                         r['lap_production'], r['lap_toll'], init_gold, init_gold + r['cum_gold']))

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
