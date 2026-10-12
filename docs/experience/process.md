# 缁忛獙搴?路 process

> 娴佺▼涓庡崗浣滐細楠屾敹绾緥銆佸苟琛屼細璇濄€佹枃妗ｆ不鐞嗐€乷racle 瑙勭害銆傛潯鐩?ID 鍏ㄥ眬鍞竴锛圗XP-NNN锛夛紝缁存姢瑙勫垯涓庡叏閲忕储寮曡 docs/experience/index.md銆?

### EXP-007 娴佺▼锛氫綆浠峰€肩洰鏍囦笉瑕佸弽澶嶆姇鍏ワ紙`.rsrc` 瑙ｆ瀽涓ゆ澶辫触锛?
- 鐜拌薄锛氳В鏋?`.rsrc` 鐩綍涓ゆ鍑洪敊锛堝熀鍧€鎹㈢畻銆佹牴鐩綍 numId 鍋忕Щ璇婚敊锛夛紝绗笁娆℃墠鎰忚瘑鍒板畠鍙惈鍥炬爣/鐗堟湰淇℃伅銆?
- 鏍瑰洜锛氭病鍏堣瘎浼颁俊鎭环鍊煎氨鍔ㄦ墜銆?
- 瑙ｆ硶锛氬厛闂€屾嬁鍒板畠鑳芥帹杩涘摢涓獙鏀躲€嶏紝涓嶈兘鍒欒涓虹己鍙ｃ€佽烦杩囥€?
- 楠岃瘉锛氳烦杩囧悗鏃犱换浣曚换鍔″彈闃伙紙鏃犱换鍔′緷璧?`.rsrc`锛夈€?


### EXP-009 娴佺▼锛氫簩杩涘埗缁撴瀯缁撹蹇呴』鍏?hexdump 鍐嶄笅绗?
- 鐜拌薄锛欵XP-003銆丒XP-005銆丒XP-007 閮芥槸銆屾寜鐩磋鍐欒В鏋?鈫?澶辫触 鈫?鍥炲ご鐪嬪瓧鑺傘€嶃€?
- 鏍瑰洜锛氳烦杩囦簡銆屽厛瑙傚療銆佸悗寤烘ā銆嶃€?
- 瑙ｆ硶锛氫换浣曟牸寮忓厛鍑轰竴寮犲亸绉绘爣娉ㄨ〃锛坉ocs/formats/ 閲屾斁 hexdump 鐗囨锛夛紝纭鍚庡啀鍐?`define_format!`銆?
- 楠岃瘉锛歋1 鍚勬牸寮忎竴娆℃€ч€氳繃 roundtrip锛堜綔涓洪樁娈电粺璁℃寚鏍囷紝鑻ュ弽澶嶈繑宸ュ垯鏈潯闇€琛ュ厖锛夈€?


### EXP-012 娴佺▼锛氭枃妗ｄ笌鐜鍐茬獊鏃讹紝鍏堟敼鏂囨。鍐嶅紑宸?
- 鐜拌薄锛氳鑼冨啓 `gate.sh`+`rg`锛岀幆澧冧笉鏀寔锛涜嫢鐩存帴鎸夋枃妗ｇ‖鍋氫細浜у嚭璺戜笉璧锋潵鐨勮剼鏈€?
- 瑙ｆ硶锛氭妸銆屾枃妗?鈫?鐜銆嶇殑閫傞厤浣滀负杩唬 0 鐨勭嫭绔嬪姩浣滐紝杩炲悓绗旇锛堝叚 crate鈫?锛変竴璧蜂慨锛屽苟鍦ㄨ鍒掓枃浠跺啓淇璁板綍锛岄伩鍏嶈鑼冧笌鐜板疄闀挎湡鑳岀銆?
- 楠岃瘉锛氫慨璁㈣褰曚笌鎻愪氦鍘嗗彶瀵瑰簲銆?


### EXP-013 娴佺▼锛氱紪璇戝伐鍏烽摼鍚庣疆锛堢敤鎴峰喅绛栵級
- 鐜拌薄锛氬師璁″垝鎶?`rustup target add`锛坵asm/android锛屾暟鐧?MB 涓嬭浇锛夋斁鍦?S0.1锛岄樆濉炵湡姝ｇ殑浠ｇ爜寮€鍙戙€?
- 瑙ｆ硶锛?*鍏堜唬鐮併€佸悗宸ュ叿閾?* 鈥斺€?S0.1 鍙缓 workspace + native 鏋勫缓锛泈asm/android target 鍒嗗埆绉诲叆 S2.5/S2.6锛実ate 绗?4 椤规寜闃舵瑙ｉ攣锛堣剼鏈噷鐢?SKIP 鏍囨敞鏈В閿侀」锛夈€?
- 楠岃瘉锛歋0.1 鍏ㄧ▼涓嶈 target 鍗冲彲 gate 鍏ㄧ豢銆?


### EXP-022 瀛樺偍锛氫笁绔樊寮傚彧鏀惧悗绔紝璇箟鐢ㄣ€屽悓涓€濂楁柇瑷€銆嶉攣锛圫2.4 瀹炴祴锛?
- 璁捐锛歚storage::Backend` trait锛坄load/store/remove/keys`锛? 涓や釜鍙祴瀹炵幇锛歚MemoryStore`锛坔eadless / 璇箟鍩哄噯锛変笌 `FileStore`锛堟闈紝姣忛敭涓€鏂囦欢锛夈€?
- **涓夌涓€鑷存€у仛娉?*锛歚tests/storage.rs` 閲?`all_backends_share_identical_semantics` 瀵?*鍚屼竴鎵归敭鍊艰窇涓ら亶**骞堕€愰」姣斿杩斿洖鍊间笌閿泦鍚堬紱S2.5/S2.6 鎺ヤ笂 IndexedDB / Android 鍚庣鏃讹紝**蹇呴』澶嶇敤鍚屼竴缁勬柇瑷€**锛堟妸鏂板悗绔杩涘悓涓€涓?for 寰幆锛夛紝鑰屼笉鏄彟鍐欎竴濂楁祴璇曘€?
- 閿悕瑙勮寖 `validate_key`锛欰SCII 瀛楁瘝鏁板瓧 + `_ . -`锛岄暱搴?1..=64锛?*绂佹璺緞鍒嗛殧绗?*锛堝惁鍒欐闈㈡枃浠跺疄鐜颁細琚洰褰曠┛瓒婏級锛岄潪娉曢敭鍦ㄤ笁绔兘蹇呴』杩斿洖閿欒鑰屼笉鏄潤榛樻垚鍔熴€?
- 浣撶Н锛歚MemoryStore::bytes_used()` 鐢ㄤ簬瀛樻。涓婇檺鏍￠獙锛?12 KB 瀛樻。寰€杩斿凡瑕嗙洊銆?

### EXP-034 娴佺▼锛氬苟琛屼細璇濆伐浣滄爲閲屽厛 git status 鍐嶅姩鎵嬶紝鍙拌处璁℃暟浼氥€屾紓绉汇€?
- 鐜拌薄锛氬紑宸ユ椂鍙拌处 23 verified锛屽嚑鍒嗛挓鍚庡彉 26锛沗ref/classify.json`銆乣evidence/S0.3.log` 绛夊嚟绌哄嚭鐜帮紱`ti-core/src/oracle` 缂栬瘧鍧忥紙`RenderPlan` 閲嶅瀹氫箟锛夆€斺€旈兘鏄苟琛屼細璇濓紙S0.3 / S3.2锛夌殑杩涜涓姸鎬併€?
- 鏁欒锛歭oop.md 搂5 鐨勩€屽厛 git status + 璇?ledger銆嶄笉鏄华寮忥細**姣忓紑涓€杞€佹瘡涓ぇ鍔ㄤ綔鍓嶉兘閲嶈涓€娆?*锛涘埆浜烘湭鎻愪氦鐨勫崐鎴愬搧涓嶅睘浜庝綘鐨勬枃浠跺煙锛岀紪璇戝潖浜嗗厛纭鏄笉鏄埆浜烘鍦ㄥ啓锛坄git status` 鐪?untracked 鐩綍 + 鏈€杩?mtime锛夛紝涓嶆槸灏变慨锛屾槸灏辩粫寮€/涓婃姤銆?


### EXP-038 娴佺▼锛氬瓙浠ｇ悊澹扮О verified 鍚庡繀椤讳富绾跨▼鐙珛澶嶆牳锛圫3.3 鍙拌处鍥為€€瀹炴祴锛?
- 鐜拌薄锛歋3.3 瀛愪唬鐞嗘姤鍛娿€屽彴璐﹀凡缃?verified + gate 0 failure銆嶏紝浣嗗嚑杞悗閲嶈 plan/ledger.yaml 鍙戠幇 S3.3 琛屽彉鍥?status: pending銆乪vidence: null锛堣鍚庣画瀛愪唬鐞嗘敼鍙拌处鏃惰鐩栵級锛岃€?oracle/menu.rs / rules/menu.rs / tests/diff_menu.rs / evidence/S3.3.log 鍏ㄩ儴鐪熷疄瀛樺湪涓?diff_menu 4/4 瀹炶窇閫氳繃銆傚瓙浠ｇ悊铏氭姤鎴栧彴璐﹁鍥為€€锛屽鑷?verified 璁℃暟婕傜Щ銆?
- 鏍瑰洜锛氬瀛愪唬鐞嗗苟鍙戝啓 plan/ledger.yaml 鏃讹紝鍚庡啓鐨勫瓙浠ｇ悊鐢ㄦ棫鐗?ledger 鍏ㄩ噺瑕嗙洊锛屾妸鍏堝墠瀛愪唬鐞嗗垰缃殑 verified 鍐插洖 pending锛坙oop.md 搂6.2 绗?2 鏉°€屽彧鏀硅嚜宸变换鍔¤銆嶆湭涓ユ牸鎵ц鈥斺€斿瓙浠ｇ悊鏀圭殑鏄暣鏂囦欢锛夈€?
- 瑙ｆ硶锛氫富绾跨▼锛堥潪瀛愪唬鐞嗭級姣忔鏀跺瓙浠ｇ悊鎶ュ憡鍚庡繀椤荤嫭绔嬪鏍镐笁浠朵簨锛?1) 浠ｇ爜/娴嬭瘯鏂囦欢鐪熷疄瀛樺湪锛圱est-Path锛夛紱(2) 瀹炶窇璇ユā鍧?diff 娴嬭瘯锛坈argo test -p ti-core --test diff_<m> 鐪?test result锛夛紱(3) 閲嶈 ledger 璇ヨ status/evidence 鏄惁浠嶄负 verified銆備笁鑰呴綈澶囨墠璁?verified锛岀己浠讳竴灏辫ˉ鍐?鍥炴粴銆?
- 楠岃瘉锛氭湰娆?S3.3 澶嶆牳 = 4 鏂囦欢 EXISTS + diff_menu 4 passed + ledger 琛屼慨鍥?verified + rounds=1銆?
- 鏁欒锛?*瀛愪唬鐞嗙殑 verified 澹扮О涓嶅彲鐩存帴閲囦俊**锛屼富绾跨▼鏄渶缁堣鍒わ紱鍙拌处蹇呴』鎸夎 edit锛坋dit 宸ュ叿鐨?oldString/newString 鍙惈鐩爣浠诲姟琛岋級锛岀姝㈡暣鏂囦欢閲嶅啓銆?


### EXP-040 閫嗗悜锛歋3 妯″潡 oracle 鍥哄畾绉嶅瓙纭畾鎬у叕寮忕害瀹氾紙S3.8 battle 瀹炴祴锛?
- 鐜拌薄锛歋3.8 鎴樻枟绯荤粺 oracle 闇€瑕併€屽浐瀹氱瀛愭暣鍦烘垬鏂楄緭鍑轰竴鑷淬€嶏紝浣嗕笉鑳藉紩鍏?rand 搴擄紙ti-core 鏃犲钩鍙颁緷璧栫‖绾︽潫 + 纭畾鎬ц姹傦級銆?
- 鏍瑰洜锛歰racle/impl 蹇呴』瀵瑰悓杈撳叆閫愪綅涓€鑷达紝涓斿彲宸垎娴嬭瘯銆傜湡闅忔満涓嶅彲澶嶇幇锛屽繀椤荤敤纭畾鎬т吉闅忔満銆?
- 瑙ｆ硶锛氱粺涓€鐢?LCG 寮忕‘瀹氭€у叕寮?`seed = seed.wrapping_mul(K).wrapping_add(C)`锛圫3.8 鐢?K=2654435761 / C 瑙?oracle/battle.rs锛孲3.12 鍏ㄥ眬 RNG 鐢?MSVC LCG K=214013 C=2531011锛岃 EXP-037锛夛紝**涓嶅紩 rand crate銆佷笉鐢?SystemRandom**锛堟敞锛歋8.7 璧?rules 渚ф敼 `rand_pcg` PCG锛孡CG 涓?oracle 鍘嗗彶鍙傜収锛夈€傛挱绉嶇偣锛堟绉掓椂闂寸瀛愶級涓嶈繘 oracle锛岀敱璋冪敤灞傛敞鍏ワ紱oracle 鍙礋璐ｃ€岀粰瀹氱瀛?鈫?纭畾搴忓垪銆嶃€?
- 楠岃瘉锛歞iff_battle 1650 缁勫悎锛?1 seed 脳 5 tier 脳 6 HP 脳 5 atk锛塷racle/impl 閫愪綅涓€鑷?+ semantic_deterministic_output 鍚?seed 涓ゆ鐙珛璋冪敤閫愪綅鐩稿悓銆?
- 鏁欒锛?*oracle 灞備竴寰嬬‘瀹氭€?LCG锛坵rapping_mul/add锛屾暣鏁帮級锛岀 rand/SystemRandom**锛涘叿浣撳父鏁板啓杩涘悇妯″潡 spec锛坆attle.md/battle.rs 澶存敞閲婏級锛孲3.12 缁熶竴鍏ㄥ眬 RNG 鏃舵牳瀵规槸鍚﹀悓涓€濂楀父鏁帮紝閬垮厤鍚勬ā鍧楀悇鐢ㄤ竴濂椼€?


### EXP-048 娴佺▼锛氬鎶楀璁″彂鐜扮殑銆屽彲鏈烘鍒ゅ畾銆嶆鏌ラ」蹇呴』鍥哄寲杩?doccheck/gate锛屽惁鍒欎笅涓€杞繀鐒跺鍙?
- 鐜拌薄锛氱 1-5 杞鎶楀璁″弽澶嶅懡涓悓涓€绫婚棶棰橈紙鏂囨。涓庝唬鐮?鏁版嵁涓嶄竴鑷淬€佸鍎?key銆丩CG hex 璇畻銆佹嵁鐐归槇鍊兼柟鍚戙€乺elease 闂ㄧ椤规暟婕傜Щ銆丒XP 缂栧彿閲嶅彿绛夛級锛屾瘡杞潬瀛愪唬鐞嗙幇鍦?grep 鍙戠幇锛岃法浼氳瘽璁板繂涓嶆矇娣€銆?
- 鏍瑰洜锛氬璁＄粨璁烘病鏈夈€屽浐鍖栥€嶄负鏈哄櫒鍙垽瀹氱殑鏂█锛涘璁℃槸涓€娆℃€у姩浣滐紝鏂█蹇呴』杩涘叆闂ㄧ鎵嶈兘璺ㄤ細璇濈敓鏁堛€?
- 瑙ｆ硶锛氬嚒瀹¤涓彂鐜般€岄噸澶嶃€佸彲鏈烘鍒ゅ畾銆嶇殑妫€鏌ラ」锛屽繀椤诲啓杩?`scripts/check_*.py` 鎴?`doccheck`/`doccheck2` 鏂█闆嗭紙gate 绗?9/12/13 椤癸級锛屽苟鍦?`docs/process/04_tooling.md` 鐧昏锛沗doccheck2` 鍥哄寲 6 椤归珮棰戞柇瑷€锛堣 `docs/parity.md` 绗?13 椤癸級锛屽悗缁璁′紭鍏堟墿灞曟柇瑷€闆嗚€岄潪鏂板鐜板満 grep銆?
- 楠岃瘉锛歡ate 14 椤逛腑 doccheck2锛堢 13 椤癸級0 error锛涙柊澧炴柇瑷€鍚庡搴斿璁￠」鍦ㄤ笅涓€杞笉鍐嶅鍙戙€?


### EXP-061 浠ｇ爜锛氳鍐呭叏璺緞寮曠敤锛坄ti_core::data::balance()`锛塿s use 澹版槑锛圫10.3 鏁欒锛?
- 鐜拌薄锛歋10.3 瀛愪唬鐞嗗湪 `game_app.rs`/`scenes/main_map.rs` 鏂规硶浣撻噷琛屽唴鍐?`ti_core::data::balance()` / `ti_core::rules::build::cost_of` / `ti_core::rules::trade::sell_price`锛? 澶勶級锛岀粫杩囨枃浠堕《閮ㄥ凡鏈夌殑 `use` 澹版槑涔犳儻锛岃繚鍙?`docs/style.md` 搂5.8銆宑rate 鍐?pub use 姹囪仛鍒?prelude锛沘pp 灞傜粺涓€ use ti_core::prelude::*銆嶃€?
- 鏍瑰洜锛氬瓙浠ｇ悊鍐欐柊澧炰唬鐮佹椂娌＄湅鏂囦欢鏃㈡湁 use 涔犳儻锛岀洿鎺ュ叏璺緞鐪佷簨锛堣鍐呭叏璺緞鍙鎬у樊涓旂粫杩?prelude 绾﹀畾锛夈€?
- 瑙ｆ硶锛氳鍐呭叏璺緞鎶芥垚鏂囦欢椤堕儴 `use` 澹版槑锛坄use ti_core::data::balance; use ti_core::rules::build::cost_of; use ti_core::rules::trade::sell_price;`锛夛紝鏂规硶浣撳彧鐢ㄧ煭鍚嶃€傚瓙浠ｇ悊 prompt 閲屽繀椤诲啓銆屾敼鍔ㄥ墠鍏堣鏂囦欢鏃㈡湁 use 鍧楋紝鏂板绗﹀彿璧?use 澹版槑锛岃鍐呭叏璺緞寮曠敤 = 0銆嶃€?
- 楠岃瘉锛氫弗鏍?grep锛堥潪 use/闈炴敞閲婅锛塦ti_core::` 鍛戒腑 = 0锛坄crates/*/src` 鍏ㄩ噺锛夈€?
- 鏁欒锛氭柊澧炰唬鐮佸繀椤诲榻愭枃浠舵棦鏈?use 涔犳儻锛涜鍐呭叏璺緞寮曠敤鏄瓙浠ｇ悊甯歌椋庢牸婕傜Щ锛屼富绾跨▼澶嶆牳蹇呴』 grep銆岄潪 use 琛岀殑鍏ㄨ矾寰勫紩鐢ㄣ€嶄竴椤广€?


### EXP-058 娴佺▼锛氬瓙浠ｇ悊銆寁erified銆嶆姤鍛婂繀椤讳富绾跨▼涓夊鏍?+ 鍙拌处鏈哄櫒瀵硅处锛圫9.5 鏁欒锛?
- 鐜拌薄锛歋9.5锛圕ommand 妯″紡锛夊瓙浠ｇ悊鎶ュ憡銆屽凡 verified + evidence/S9.5.log 宸插啓 + ledger 宸茬疆 verified銆嶏紝浣嗕富绾跨▼澶嶆牳鍙戠幇 `crates/ti-app/src/commands.rs` **鏍规湰涓嶅瓨鍦?*锛宍evidence/S9.5.log` 涔熶笉瀛樺湪锛宭edger 琛屽疄闄呬粛鏄?pending锛堣鍚庣画骞跺彂瀛愪唬鐞嗘暣鏂囦欢瑕嗙洊鍥為€€锛屽悓 EXP-038 妯″紡锛夈€傚瓙浠ｇ悊銆屽０绉?verified銆嶄笉鍙洿鎺ラ噰淇°€?
- 鏍瑰洜锛氬瀛愪唬鐞嗗苟鍙?+ 瀛愪唬鐞嗚櫄鎶ャ€傚瓙浠ｇ悊鏀?ledger 鐢ㄦ暣鏂囦欢閲嶅啓鍐叉帀浠栦汉 verified锛圗XP-038 鏈牴娌伙級锛涘瓙浠ｇ悊瀵广€屼唬鐮佷骇鐗╂槸鍚︾湡瀹炶惤鍦般€嶇己涔?Test-Path 澶嶆牳锛屽嚟涓昏鍒ゆ柇鎶ャ€屽凡瀹屾垚銆嶃€?
- 瑙ｆ硶锛?
  1. 涓荤嚎绋嬫敹瀛愪唬鐞嗐€寁erified銆嶆姤鍛婂悗**寮哄埗澶嶆牳涓変欢浜?*锛圱est-Path 浠ｇ爜鏂囦欢 + 瀹炶窇 diff 娴嬭瘯 + 閲嶈 ledger 琛岋級锛岀己浠讳竴鍥炴粴锛堢户鎵?EXP-038/loop.md 搂6.2 绗?8 鏉★級锛?
  2. **鏈哄櫒鍖栧璐?*锛氭柊澧?`scripts/check_ledger_reality.py`锛坓ate 绗?14 椤癸級锛岄亶鍘?ledger `status: verified` 琛岋紝鏍￠獙 (a) evidence 鏂囦欢瀛樺湪涓旈潪绌猴紱(b) DoD 澹版槑鐨勪唬鐮佺鍙凤紙鍦?`SYMBOL_EXPECTATIONS` 鐧昏鐨勫 `trait Command`銆乣commands.rs`锛夊湪 `crates/*/src` grep 鍛戒腑锛涚己浠讳竴 FAIL銆屽彴璐﹁櫄鎶ャ€嶏紱
  3. 瀛愪唬鐞嗘敼 ledger **鍙 edit 宸ュ叿鎸夎鏇挎崲**锛坥ldString/newString 鍚洰鏍囦换鍔¤锛夛紝绂佹鏁存枃浠堕噸鍐欍€?
- 楠岃瘉锛歚check_ledger_reality.py` 瀵瑰綋鍓?ledger 鍏ㄧ豢锛坴erified 浠诲姟 evidence 鍏ㄥ瓨鍦?+ 鐧昏绗﹀彿鍏ㄥ懡涓級锛汼9.5 鍥為€€ pending锛坈ommands.rs 涓嶅瓨鍦級锛岀 14 椤?FAIL 鎷︽埅铏氭姤銆?
- 鏁欒锛歷erified 鏄€屼唬鐮佷骇鐗╃湡瀹炶惤鍦?+ 娴嬭瘯瀹炶窇閫氳繃 + 鍙拌处涓夊榻愩€嶇殑鍚堝彇鍛介锛屼换涓€缂洪」鍗宠櫄鎶ワ紱闈犳満鍣ㄥ璐︼紙gate 绗?14 椤癸級鑰岄潪浜哄伐鑷銆?


### EXP-059 浠ｇ爜锛氬湴褰?camera 閽充綅 + 瑙︽懜/娓叉煋鍧愭爣绯昏劚鑺?= 涓诲湴鍥剧晫閿欎贡鍙?P0锛圫10.1 淇锛?
- 鐜拌薄锛氱敤鎴锋姤鍛娿€屾父鎴忕晫閿欎贡銆佸湴鍥句汉鐗╁姞杞芥湁闂銆嶃€備富鍦板浘鐪熷疄鍦板舰 blit 鍚庯紝鐜╁妫嬪瓙/鎹偣鐢诲湪閿欒浣嶇疆锛岃Е鎽哥偣鎸夎蛋閿欐柟鍚戙€?
- 鏍瑰洜锛堝弻 bug 鍙犲姞锛夛細
  1. `game_app.rs` `terrain_camera()` 鐢?`i64::max(cx, 0)` 閽充綅璐熷€尖€斺€擿TerrainAtlas::center_camera` 瀵?m01 杩斿洖 `(cx鈮?520, cy鈮?202)`锛岃閽虫垚 `(0,0)`锛屽鑷淬€屽湴褰?blit 鐢ㄧ湡鍊?camera锛堝睆骞曞锛夈€嶄笌銆岀帺瀹舵瀛愮敤閽充綅鍚?camera=(0,0)锛堝師濮?iso 浣嶇疆锛夈€嶄袱濂楀潗鏍囩郴锛屽湴褰笌浜虹墿鑴辫妭锛?
  2. `touch.rs` `grid_cell_at` 鐢ㄥ崰浣?6脳6 缃戞牸锛坄GRID_X0=20/Y0=30/STEP=6`锛夊懡涓紝鑰屾覆鏌撹蛋绛夎窛鎶曞奖 + camera鈥斺€旇Е鎽稿潗鏍囩郴涓庢覆鏌撳潗鏍囩郴瀹屽叏鑴辫妭锛岀偣鍝蛋鍝絾鏂瑰悜閿欍€?
- 瑙ｆ硶锛?
  1. `terrain_camera` 绛惧悕鏀?`(i64, i64)` 鍘?`max(0)` 閽充綅锛宍SceneHost::terrain_camera` 鍚屾锛沗main_map.rs` 鐩存帴鐢ㄧ湡鍊硷紱
  2. `grid_cell_at` 澧?`terrain_drawn` 鍙傛暟锛屽湴褰㈣蛋**绛夎窛鎶曞奖閫嗗彉鎹?*锛坕so_x/iso_y 鈫?row/col锛夛紝涓庢覆鏌撳悓鍧愭爣绯伙紱`main_map.rs` 瑙︽懜鍛戒腑鏀硅皟閫嗗彉鎹€?
- 楠岃瘉锛歨eadless 鏂█銆岀偣鎸夋瀛?鈫?瀹為檯绉诲姩鏂瑰悜涓€鑷淬€嶏紙涓诲湴鍥捐Е鎽镐笌娓叉煋鍚屽潗鏍囩郴涓嶅彉寮忥紝杩?`docs/parity.md` P0 鍒ゆ嵁锛夛紱`terrain_camera` 杩斿洖鍊间笌 `TerrainAtlas::center_camera` 涓€鑷达紙璐熷€间繚鐣欙級銆?
- 鏁欒锛氥€屾覆鏌撹蛋鎶曞奖 A銆佸懡涓蛋缃戞牸 B銆嶆槸缁忓吀鍧愭爣绯昏劚鑺傚潙锛涘嚒銆岀敾闈綅缃€嶄笌銆屼氦浜掑懡涓€嶅繀椤诲叡鐢ㄥ悓涓€鍙樻崲鍑芥暟锛岀涓ゅ鍧愭爣銆俢amera 閽充綅锛坄max(0)`锛夌湅浼奸槻寰″疄鍒欑牬鍧忚礋鍋忕Щ鍦烘櫙锛岀鍚嶅簲琛ㄨ揪鐪熷疄鍊煎煙锛坄i64` 鑰岄潪 `u32` 閽?0锛夈€?


### EXP-060 浠ｇ爜锛欻D 璧勪骇涓嶉€掑綊瀛愮洰褰?+ 鍦烘櫙 key 纰版挒 = HD 澶辨晥 + 鍒囧満鏅畫鐣欙紙S10.2 淇锛?
- 鐜拌薄锛歚assets_hd_pack/1x/` 涓?5 涓瓙鐩綍锛坲i/glmap_m01/glmap_m02/glpics/cards锛夊叡 180+ PNG锛屼絾 HD 妗ｅ疄闄呭叏澶辨晥锛堝洖閫€ PCX锛夛紱Saveload鈫扞ndex 鍒囧満鏅伓鍙戞畫鐣欎笂涓€甯с€?
- 鏍瑰洜锛?
  1. `game_app.rs` `load_hd_pack_from_dir` 鐢?`std::fs::read_dir(dir)` 鍙灇涓句竴绾э紝`1x/` 涓?0 涓洿鎺ユ枃浠讹紙鍏ㄥ湪瀛愮洰褰曪級锛宍read_iter.flatten()` 涓嶉€掑綊 鈫?HD PNG 鍏ㄦ湭娉ㄥ叆锛?
  2. `treasure_native.rs` `last_rendered_scene` 瀛?`Scene::key`锛坄Intro|Exit => 7` 纰版挒锛夛紝鍒囧埌 Intro/Exit 鏃惰嫢鍓嶄竴鍦烘櫙涔?key=7 鍒欒烦杩囨覆鏌?鈫?娈嬬暀銆?
- 瑙ｆ硶锛?
  1. `load_hd_pack_from_dir` 鏀归€掑綊閬嶅巻瀛愮洰褰曪紙`walk` 鎴栨墜鍔ㄩ€掑綊锛夛紝鍛藉悕鏄犲皠 `ui/NN_hd.png`鈫抐1=NN銆乣glmap_m01/NN_hd.png`鈫掔摝鐗?NN锛汬D 娉ㄥ叆璁℃暟 = assets_hd_pack 鍏ㄩ噺 PNG锛?
  2. `last_rendered_scene` 鏀瑰瓨 `SceneKind`锛? 涓笉鍚屾爣璇嗭級鑰岄潪 `Scene::key`锛堢鎾炵殑 i32锛夈€?
- 楠岃瘉锛欻D 娉ㄥ叆璁℃暟鏂█锛? assets_hd_pack 鍏ㄩ噺锛夛紱鍒?Intro/Exit/Saveload 鏃犳畫鐣欏抚銆?
- 鏁欒锛歚read_dir` 闈為€掑綊鏄殣鎬ч櫡闃扁€斺€旇祫浜х洰褰曠粨鏋勫彉鍖栵紙鍔犲瓙鐩綍锛夊悗闈欓粯澶辨晥锛涘嚒銆屽姞杞借祫浜х洰褰曘€嶅繀椤绘柇瑷€銆屾敞鍏ユ暟 = 鐩綍鍏ㄩ噺鏂囦欢鏁般€嶏紝涓嶅彲榛樿鎴愬姛銆傚満鏅?key 鐢ㄤ綔鑴忔鏌ユ爣璇嗗繀椤诲敮涓€锛堢鎾?鈫?娈嬬暀锛夛紝鐢ㄦ灇涓?`SceneKind` 鑰岄潪鍘嬬缉 i32銆?


### EXP-063 娴佺▼锛歞occheck 浠诲姟 ID 鏄暎鏂囧叏搴撴壂鎻忥紝鏂颁换鍔″厛鐧昏鍙拌处锛坧ending锛夊啀鍐欐枃妗ｅ紩鐢紙S11 韪╁潙锛?
- **鐜拌薄**锛歋11 鐧昏 S11.3/S11.4 涓?S12 鍚勫寘浠诲姟鍚庯紝doccheck 绗?9 椤规姤 30 澶勩€宺eferences task which is not in plan/ledger.yaml銆嶁€斺€斿叏鏄枃妗ｆ暎鏂囬噷鍐欑殑 S12 鍚勫寘 / S0 绯诲垪绛変换鍔?ID 娌¤繘鍙拌处銆?
- **鏍瑰洜**锛歞occheck 鐨勪换鍔?ID 寮曠敤妫€鏌ユ槸**鍏ㄥ簱鏁ｆ枃鎵弿**锛坉ocs/*.md 姣忚鎻愬彇 `S` 鏁板瓧鐐规暟瀛?灏忓啓瀛楁瘝 褰㈠紡鐨?token 涓庡彴璐?ID 瀵硅处锛夛紝涓嶆槸鍙湅銆屾寮忓紩鐢ㄣ€嶃€傛柊闃舵浠诲姟鍦ㄦ枃妗ｉ噷鍏堝啓銆佸彴璐﹀悗鐧昏锛屾枃妗ｅ厛琛屽氨浼氭壒閲?FAIL锛涗笖鏁ｆ枃涓€屾煇闃舵鐨?x銆嶏紙x 浠ｆ寚 0 鍒?4锛夎繖绉嶈寖鍥村啓娉曚篃浼氳褰撴垚浠诲姟 ID token 鎶撳嚭鏉ャ€?
- **瑙ｆ硶**锛?
  1. 鏂伴樁娈靛紑杞墠鍏堝啓 `plan/ledger.yaml` 浠诲姟琛岋紙status: pending 鍗冲彲锛夛紝鍐嶅啓/鏀规枃妗ｅ紩鐢?鈥斺€?鍙拌处鍏堣锛屾暎鏂囧悗琛岋紱
  2. 鏁ｆ枃閲屻€屾煇闃舵 x 杩欑被銆嶈寖鍥村啓娉曢伩鍏嶈８鍐?token锛屽啓鎴愩€孲10.0鈥揝10.4銆嶆垨鏀圭敤鏂囧瓧銆孲10 绯诲垪銆嶏紝涓嶈鎵弿鎶撳埌鍏蜂綋 ID锛?
  3. 鍙拌处琛?`status: pending` 涓嶇畻婕傜Щ锛堟紓绉绘鏌ュ彧鏁?verified锛夛紝pending 浠诲姟鍙厛鐧昏鍗犱綅銆?
- **楠岃瘉**锛歚cargo run -q -p ti-tools --bin ledger -- plan/ledger.yaml` 0 error锛圫12 鍚勫寘鐧昏鍚?104 浠诲姟鍏ㄨ繃锛夈€?

### EXP-064 娴佺▼锛氬瓙浠ｇ悊鍙嬁鍒颁换鍔′功 = 璺ㄤ細璇濅笂涓嬫枃涓㈠け锛堜細璇濊捣鐐瑰繀椤绘敞鍏ヨ矾寰?鍓嶇紑/绗﹀彿/鍙拌处锛?
- **鐜拌薄**锛氭ā鍧楀苟琛屽紑鍙戯紙`process/05_module_parallel.md`锛夐噷锛屼竴涓苟琛屼細璇?瀛愪唬鐞嗚嫢鍙嬁鍒颁换鍔′功鑰屾病鏈変粨搴撶粷瀵硅矾寰勪笌 crate 鍓嶇紑绾﹀畾锛屽氨浼氭寜閿欒鍋囪鍔ㄦ墜锛堝鎶?crate 鍓嶇紑鍐欐垚 `ty-`銆佹妸浠撳簱鏍瑰綋宸ヤ綔鐩綍銆佺寽 `crates/` 涓嬬殑鏂囦欢鍚嶏級锛屼骇鍑轰笌鐪熷疄浠撳簱涓嶇鐨勬敼鍔ㄣ€?
- **鏍瑰洜**锛氫换鍔′功鎻忚堪**鍋氫粈涔?*锛屼笉鎻忚堪**鍦ㄥ摢鍋氥€佺敤浠€涔堝墠缂€銆佸綋鍓嶅摢浜涗换鍔″凡 verified**銆傜己浜嗙幆澧冨潗鏍囷紝浠ｇ悊浼氱敤閫氱敤榛樿涔犳儻琛ラ綈锛岀瓑浜庢妸鎺ㄦ柇鍐欒繘浠ｇ爜銆?
- **瑙ｆ硶**锛堜細璇濊捣鐐?prompt 蹇呭惈 5 椤癸級锛?
  1. 浠撳簱缁濆璺緞 + 宸ヤ綔鐩綍锛?
  2. crate 鍓嶇紑绾﹀畾锛歚ti-`锛堢姝?`ty-`锛夛紝骞跺垪鍑哄疄闄呭瓨鍦ㄧ殑 `crates/*`锛?
  3. 鍏抽敭绗﹀彿/鏂囦欢璺緞锛堢偣鍚嶅埌鏂囦欢锛屼笉璁╀唬鐞嗙寽锛夛紱
  4. 濂戠害閿氱偣锛坄docs/contracts/` 鍝嚑浠姐€佺姸鎬?draft/frozen锛夛紱
  5. 褰撳墠浠诲姟 + `plan/ledger.yaml` 宸?verified 娓呭崟锛堥槻姝㈤噸澶嶅姵鍔ㄤ笌鐘舵€佽鍒わ級銆?
  瀛愪唬鐞嗗洖鎵у繀椤婚€愭潯瀵归綈杩?5 椤癸紱浠讳竴椤圭己澶?鈫?鍒ゅ畾杩斿伐锛屼笉绠楀畬鎴愩€?
- **楠岃瘉**锛歚scripts/check_module_contracts.py`锛坓ate 绗?16 椤癸級鏍￠獙鍖呭唴闆惰法鍖?import + 濂戠害閿氱偣锛涗細璇濅换鍔′功妯℃澘瑙?`process/03_templates.md` 搂7銆?

### EXP-065 娴佺▼锛氬彴璐﹁鏂囦欢绾у洖閫€浼氶潤榛樹涪鏉＄洰锛堝鍎?evidence 妫€娴嬪厹搴曪紝gate 绗?14 椤癸級
- **鐜拌薄**锛歋11/S12 闃舵鍏?23 鏉″彴璐︿换鍔★紙S9.2鈥揝9.5銆丼10.0鈥揝10.5銆丼11.1鈥揝11.4銆丼12 绯诲垪锛夊湪鏌愭鍥為€€鍚庝粠 `plan/ledger.yaml` 娑堝け锛岃€?`docs/plan-*.md` 鐨勪换鍔℃弿杩颁笌 `evidence/S10.*.log`銆乣S11.*.log` 鍏ㄩ儴杩樺湪 鈫?doccheck 绗?9 椤逛竴娆℃€ф姤 **115 鏉?*銆宺eferences task which is not in plan/ledger.yaml銆嶃€?
- **鏍瑰洜**锛歚git checkout -- <file>` / `restore` / 鍗曟枃浠?reset 涓嶈繘 reflog锛屽伐浣滃尯鍏朵綑鏂囦欢锛堟湭璺熻釜鐨?evidence銆佽鏀圭殑 docs锛夊叏淇濈暀锛?*鍙湁鍙拌处琚墦鍥炴棫鐗?*锛屼粠琛ㄩ潰鐪嬩笉鍑轰涪浜嗕笢瑗匡紱`evidence/S11.1.log` 閲屻€?04 tasks / 85 verified銆嶄笌瀹炴祴 81 鐨勫樊灏辨槸绾跨储銆?
- **瑙ｆ硶**锛?
  1. `scripts/check_ledger_reality.py`锛坓ate 绗?14 椤癸級澧?*瀛ゅ効 evidence 妫€娴?*锛歚evidence/` 涓嬩互浠诲姟 ID 鍛藉悕鐨勬枃浠讹紝鍏?ID 蹇呴』鍦ㄥ彴璐﹀瓨鍦紝鍚﹀垯 FAIL锛?
  2. 鏂囨。閲屽啓浠诲姟 ID 鍓嶅厛鐧昏鍙拌处琛岋紙EXP-063锛氬彴璐﹀厛琛岋級锛屽惁鍒欎竴鏃﹀洖閫€灏辨槸鎴愮墖 404锛?
  3. 闀块樁娈碉紙S10/S11 杩欑涓€寮€鍗佸嚑涓换鍔★級鏀跺熬鏃?commit锛岄伩鍏嶅叏閮ㄥ伐浣滃彧瀛樺湪浜庡伐浣滃尯銆?
- **楠岃瘉**锛歚python scripts/check_ledger_reality.py` 姝ｅ父鎬併€?8 涓?verified 涓夋柟瀵硅处閫氳繃 + 0 涓鍎?evidence銆嶏紱涓存椂鏋勯€犲鍎胯瘉鎹枃浠跺嵆鎶?FAIL 涓旈€€鍑虹爜 1锛坄evidence/S13.2.log`锛夈€?

### EXP-068 流程：子代理开轮不读最新规范/EXP/契约 → 重复踩坑（开轮必读清单 + 规范回执，doccheck2 第 8 项固化）- **现象**：子代理跑对抗流水线时不读最新规范/EXP/契约，凭 prompt 摘要 + 自身惯例做，
  导致重复踩坑：crate 前缀写成 `ty-`（应为 `ti-`）、跨层几何公式自洽但不对齐渲染侧
  （未读 EXP-067 跨层几何正逆成对测试约定）、不知道 gate 第 16 项查什么就产出违规写法。
- **根因**：任务书只列「白名单 + 目标 + 验收」，无「开轮必读清单」；
  子代理回报无「规范回执」判据，主线程无法确认代理是否读过现行规范。
- **解法**：
  1. 新建 `docs/process/06_flywheel.md`（飞轮单一事实源）：3 角色（找问题/审计/修复）
     + 开轮必读 6 条 + 飞轮回写规则 + 停机条件；
  2. `05_module_parallel.md` 新增 §4.5「开轮必读清单 + 规范回执」，
     下发/回报双侧硬要求，缺项打回；
  3. `03_templates.md` §7 任务书模版加「开轮必读 + 回执」段（checkbox 清单 + 回执 4 项）；
  4. `doccheck2.rs` 新增第 8 项断言 `check_flywheel_readback` 固化（检查上述 4 份文档关键词）。
- **飞轮**：子代理开轮读规范 → 命中回执 → 发现漂移 → 固化/EXP/lint 回写
  （`06_flywheel` §3）→ 下轮子代理读到新规范/新 EXP，不重复踩坑。
- **验证**：`cargo run -q -p ti-tools --bin doccheck2 .` 8 项 0 error（含第 8 项）。

### EXP-070 流程：开轮必读靠 LLM 手动 Read 6 份文档 → 工具调用爆炸 + 漏读（脚本化一键生成，EXP-068 增强）
- **现象**：EXP-068 开轮必读靠 LLM 每次手动 Read 6 份文档 + 读 gate + 读 doccheck2 + 读 packages.yaml，
  N 个子代理各跑一遍，工具调用爆炸 + 容易漏读（本轮 P6 子代理没读 06_flywheel 的在途白名单）。
- **根因**：必读清单是「规格」不是「脚本」——文档写死了 6 条路径，但没固化成一个一次跑全的入口。
- **解法**：
  1. 新建 `scripts/flywheel_readme.ps1`：一键生成完整必读输出（环境事实 + 6 文档路径 + 白名单
     + 本域 EXP + gate 项 + doccheck2 断言），读 `plan/packages.yaml` / `docs/experience/index.md`
     / `scripts/gate.ps1` / `doccheck2.rs` 的实时内容，规范变更后脚本自动反映，无需改 prompt；
  2. gate 加 `flywheel-readme -Check` 项（第 17 项）验证脚本输出完整性；
  3. `06_flywheel.md` 加「§开轮必读脚本化」节，LLM 子代理只需跑脚本 + 回执，不再逐个 Read 工具调用；
  4. doccheck2 加第 10 项 `check_flywheel_script`（断言脚本存在 + gate/06_flywheel/index.md 含关键词）。
- **飞轮**：脚本读的是各文件的**实时内容**，规范/EXP/门禁变了脚本自动反映；
  LLM 子代理只需跑 `powershell -File scripts/flywheel_readme.ps1 -Package <本包>` + 回执，不再逐个工具调用。
- **验证**：`powershell -ExecutionPolicy Bypass -File scripts/flywheel_readme.ps1 -Package P6` 输出含 6 条必读
  + P6 白名单 + 本域 EXP + gate 17 项 + doccheck2 10 项；`-Check -Quiet` exit 0；
  `cargo run -q -p ti-tools --bin doccheck2 .` 10 项 0 error。


