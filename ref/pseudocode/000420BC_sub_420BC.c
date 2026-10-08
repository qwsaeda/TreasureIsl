/* ea:   0x000420BC */
/* name: sub_420BC */
/* size: 260 */
int __fastcall sub_420BC(_DWORD *a1, int a2, int a3)
{
  __int16 **v4; // r0
  int v5; // r0
  _WORD *v7; // r1
  _WORD *v8; // r0
  void *v9[4]; // [sp+8h] [bp-34h] BYREF
  void *v10[4]; // [sp+18h] [bp-24h] BYREF
  void *v11[5]; // [sp+28h] [bp-14h] BYREF

  a1[1] = a2;
  if ( a2 == 0 )
    return 0;
  a1[2] = a3;
  if ( a3 == 0 )
    return 0;
  sub_1333C(v10);
  sub_1333C(v9);
  sub_1309C((int)v10);
  v4 = (__int16 **)sub_37EDC(v11, (int)v10, (char *)aDataGlpicsPak_0);
  if ( v4 != (__int16 **)v9 )
    sub_17890((int)v9, *v4, v4[1]);
  sub_12F48(v11);
  if ( (unsigned __int8)sub_42390(a1, v9[0], a1 + 8) == 0 || (v5 = sub_62694(176, 220, 16, 0), a1[13] = v5, v5 == 0) )
  {
    sub_12F48(v9);
    sub_12F48(v10);
    return 0;
  }
  v7 = *(_WORD **)(v5 + 28);
  v8 = v7 + 38720;
  do
    *v7++ = -1;
  while ( v7 != v8 );
  sub_12F48(v9);
  sub_12F48(v10);
  return 1;
}

