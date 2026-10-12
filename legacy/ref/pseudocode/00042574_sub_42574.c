/* ea:   0x00042574 */
/* name: sub_42574 */
/* size: 516 */
int __fastcall sub_42574(_DWORD *a1, int a2, int a3)
{
  wchar_t *v4; // r3
  wchar_t v5; // t1
  _WORD *v6; // r0
  _WORD *v7; // r4
  __int16 v9; // r2
  __int16 **v10; // r0
  __int16 **v11; // r0
  void *v12; // r0
  int v13; // r3
  _DWORD *v14; // r4
  _DWORD *v15; // r0
  _DWORD v16[4]; // [sp+14h] [bp-7Ch] BYREF
  _DWORD v17[4]; // [sp+24h] [bp-6Ch] BYREF
  _DWORD v18[4]; // [sp+34h] [bp-5Ch] BYREF
  void *v19[4]; // [sp+44h] [bp-4Ch] BYREF
  void *v20[4]; // [sp+54h] [bp-3Ch] BYREF
  _DWORD v21[4]; // [sp+64h] [bp-2Ch] BYREF
  int v22; // [sp+74h] [bp-1Ch] BYREF

  a1[1] = a2;
  if ( a2 == 0 )
    return 0;
  a1[2] = a3;
  if ( a3 == 0 )
    return 0;
  v4 = &aIntroMap[9];
  do
  {
    v5 = v4[1];
    ++v4;
  }
  while ( v5 != 0 );
  v6 = operator new(2 * (((unsigned int)((char *)v4 - aF_0) >> 1) + 1));
  v7 = v6;
  a1[13] = v6;
  if ( v6 == nullptr )
    return 0;
  do
  {
    v9 = *(_WORD *)((char *)v7 + aF_0 - (char *)v6);
    *v7++ = v9;
  }
  while ( v9 != 0 );
  sub_1333C(v16);
  sub_1333C(v18);
  sub_1333C(v17);
  sub_1309C((int)v16);
  sub_162DC(v16, (char *)aData);
  v10 = (__int16 **)sub_37EDC(v19, (int)v16, (char *)aUiPak);
  if ( v10 != v18 )
    sub_17890((int)v18, *v10, v10[1]);
  sub_12F48(v19);
  v11 = (__int16 **)sub_37EDC(v20, (int)v16, (char *)aIntroMap);
  if ( v11 != v17 )
    sub_17890((int)v17, *v11, v11[1]);
  sub_12F48(v20);
  v12 = operator new(0x3FCu);
  if ( v12 != nullptr )
    v13 = sub_43B8C(v12, a1[2]);
  else
    v13 = 0;
  a1[12] = v13;
  if ( v13 != 0 )
  {
    sub_43D54(v13, *(_DWORD *)(a1[1] + 16));
    v14 = sub_132FC(v21, v17);
    v15 = sub_132FC(&v22, v18);
    (*(void (__fastcall **)(_DWORD, _DWORD, _DWORD, _DWORD, _DWORD, _DWORD, _DWORD))(*(_DWORD *)a1[12] + 4))(
      a1[12],
      *v15,
      v15[1],
      v15[2],
      *v14,
      v14[1],
      v14[2]);
    sub_12EA0((int)v17);
    sub_12EA0((int)v18);
    sub_12EA0((int)v16);
    return 1;
  }
  else
  {
    sub_12EA0((int)v17);
    sub_12EA0((int)v18);
    sub_12EA0((int)v16);
    return 0;
  }
}

