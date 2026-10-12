/* ea:   0x00039EEC */
/* name: sub_39EEC */
/* size: 364 */
int __fastcall sub_39EEC(_DWORD *a1)
{
  int v2; // r0
  int v4; // r0
  int v5; // r1
  int v6; // r0
  int v7; // r1
  int v8; // r0
  int v9; // r1
  int v10; // r3
  int v11; // kr00_4
  int v12; // r0
  void *v13[6]; // [sp+8h] [bp-18h] BYREF

  sub_1333C(v13);
  sub_1309C((int)v13);
  v2 = sub_12F04(aDataGlpicsPak);
  sub_134A4((int)v13, (int)aDataGlpicsPak, (int)&aDataGlpicsPak[v2]);
  if ( (unsigned __int8)sub_3A058(a1, v13[0]) != 0 )
  {
    v4 = a1[15];
    if ( v4 == 0 )
      __debugbreak();
    v5 = *(_DWORD *)(v4 + 4) >> 1;
    v6 = a1[14];
    a1[3] = v5;
    if ( v6 == 0 )
      __debugbreak();
    v7 = a1[13];
    a1[4] = *(_DWORD *)(v6 + 8);
    if ( v7 == 0 )
      __debugbreak();
    a1[5] = *(_DWORD *)(v7 + 4);
    if ( v7 == 0 )
      __debugbreak();
    v8 = *(_DWORD *)(v7 + 8);
    v9 = a1[16];
    a1[6] = v8;
    if ( v9 == 0 )
      __debugbreak();
    a1[7] = *(_DWORD *)(v9 + 4);
    if ( v9 == 0 )
      __debugbreak();
    v10 = a1[17];
    a1[8] = *(_DWORD *)(v9 + 8);
    if ( v10 == 0 )
      __debugbreak();
    a1[9] = *(_DWORD *)(v10 + 4) / 6u;
    if ( v10 == 0 )
      __debugbreak();
    v11 = 176 - a1[5];
    v12 = 220 - a1[6];
    a1[10] = *(_DWORD *)(v10 + 8);
    a1[11] = v11 / 2;
    a1[12] = v12 / 2;
    sub_12F48(v13);
    return 1;
  }
  else
  {
    sub_132A0(v13);
    return 0;
  }
}

