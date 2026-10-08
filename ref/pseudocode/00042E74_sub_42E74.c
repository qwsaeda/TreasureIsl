/* ea:   0x00042E74 */
/* name: sub_42E74 */
/* size: 288 */
void __fastcall sub_42E74(int *a1, int a2)
{
  int v4; // r0
  void *v5; // r0
  int v6; // r3
  int v7; // r0
  _DWORD *v8; // r4
  _DWORD *v9; // r0
  _DWORD v10[4]; // [sp+14h] [bp-50h] BYREF
  void *v11[4]; // [sp+24h] [bp-40h] BYREF
  _DWORD v12[4]; // [sp+34h] [bp-30h] BYREF
  int v13; // [sp+44h] [bp-20h] BYREF

  sub_1333C(v11);
  sub_1333C(v10);
  sub_1309C((int)v11);
  sub_1309C((int)v10);
  v4 = sub_12F04(aDataUiPak);
  sub_134A4((int)v11, (int)aDataUiPak, (int)&aDataUiPak[v4]);
  sub_162DC(v10, (char *)aDataMsglMap);
  v5 = operator new(0x610u);
  if ( v5 != nullptr )
    v6 = sub_462DC(v5, a1[1]);
  else
    v6 = 0;
  v7 = *a1;
  a1[4] = v6;
  sub_43D54(v6, *(_DWORD *)(v7 + 16));
  v8 = sub_132FC(v12, v10);
  v9 = sub_132FC(&v13, v11);
  (*(void (__fastcall **)(int, _DWORD, _DWORD, _DWORD, _DWORD, _DWORD, _DWORD))(*(_DWORD *)a1[4] + 4))(
    a1[4],
    *v9,
    v9[1],
    v9[2],
    *v8,
    v8[1],
    v8[2]);
  sub_4661C(a1[4], 1, 0, a1[2], a2);
  sub_12EA0((int)v10);
  sub_132A0(v11);
}

