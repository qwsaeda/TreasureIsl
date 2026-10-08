/* ea:   0x0003ECDC */
/* name: sub_3ECDC */
/* size: 616 */
int __fastcall sub_3ECDC(_DWORD *a1, int a2, int a3)
{
  __int16 **v5; // r0
  __int16 **v6; // r0
  void *v7; // r0
  int v8; // r3
  _DWORD *v9; // r4
  _DWORD *v10; // r0
  _DWORD *v11; // r0
  _DWORD v12[4]; // [sp+14h] [bp-80h] BYREF
  _DWORD v13[4]; // [sp+24h] [bp-70h] BYREF
  void *v14[4]; // [sp+34h] [bp-60h] BYREF
  void *v15[4]; // [sp+44h] [bp-50h] BYREF
  void *v16[4]; // [sp+54h] [bp-40h] BYREF
  _DWORD v17[4]; // [sp+64h] [bp-30h] BYREF
  int v18; // [sp+74h] [bp-20h] BYREF

  a1[1] = a2;
  if ( a2 == 0 )
    return 0;
  a1[2] = a3;
  if ( a3 == 0 )
    return 0;
  sub_1333C(v13);
  sub_1333C(v12);
  sub_1333C(v14);
  sub_1309C((int)v13);
  sub_162DC(v13, (char *)aData);
  v5 = (__int16 **)sub_37EDC(v15, (int)v13, (char *)aUiPak);
  if ( v5 != v12 )
    sub_17890((int)v12, *v5, v5[1]);
  sub_12F48(v15);
  v6 = (__int16 **)sub_37EDC(v16, (int)v13, (char *)aSelectroleMap);
  if ( v6 != (__int16 **)v14 )
    sub_17890((int)v14, *v6, v6[1]);
  sub_12F48(v16);
  v7 = operator new(0x3FCu);
  if ( v7 != nullptr )
    v8 = sub_43B8C(v7, a1[2]);
  else
    v8 = 0;
  a1[12] = v8;
  if ( v8 != 0 )
  {
    sub_43D54(v8, *(_DWORD *)(a1[1] + 16));
    v9 = sub_132FC(v17, v14);
    v10 = sub_132FC(&v18, v12);
    (*(void (__fastcall **)(_DWORD, _DWORD, _DWORD, _DWORD, _DWORD, _DWORD, _DWORD))(*(_DWORD *)a1[12] + 4))(
      a1[12],
      *v10,
      v10[1],
      v10[2],
      *v9,
      v9[1],
      v9[2]);
    v11 = sub_37EDC(v17, (int)v13, (char *)aGlpicsPak);
    sub_161B0((int)v12, (int)v11);
    sub_12EA0((int)v17);
    if ( (unsigned __int8)sub_3FE84(a1, v12[0], a1 + 8) != 0 )
    {
      if ( byte_81194 != 0 && byte_81195 != 0 && byte_81196 != 0 && byte_81197 != 0 )
      {
        a1[13] = 5;
        a1[14] = 58;
        a1[15] = 75;
        a1[16] = 5;
      }
      sub_12EA0((int)v14);
      sub_12EA0((int)v12);
      sub_12EA0((int)v13);
      return 1;
    }
    else
    {
      sub_12EA0((int)v14);
      sub_12EA0((int)v12);
      sub_12EA0((int)v13);
      return 0;
    }
  }
  else
  {
    sub_12F48(v14);
    sub_12EA0((int)v12);
    sub_12EA0((int)v13);
    return 0;
  }
}

