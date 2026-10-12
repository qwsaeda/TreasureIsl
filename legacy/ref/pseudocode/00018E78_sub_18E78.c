/* ea:   0x00018E78 */
/* name: sub_18E78 */
/* size: 1084 */
int __fastcall sub_18E78(int a1, int a2, int a3)
{
  __int16 v6; // r4
  int v7; // r0
  __int16 v8; // r2
  int v9; // r1
  int v10; // r4
  int v11; // [sp+0h] [bp-24h] BYREF
  int v12; // [sp+4h] [bp-20h] BYREF
  void *v13[7]; // [sp+8h] [bp-1Ch] BYREF

  if ( (unsigned __int8)sub_5F498(a2, a3, 3565, &v12) == 0 )
    return 0;
  v11 = sub_138D4((unsigned __int8)v12);
  if ( v11 == 0 )
    return 0;
  sub_1333C(v13);
  if ( (unsigned __int8)sub_5F55C(a2, a3, 4062401, v13) == 0 )
  {
    sub_12F48(v13);
    (*(void (__fastcall **)(int, int))(*(_DWORD *)v11 + 4))(v11, 1);
    return 0;
  }
  sub_161B0(v11 + 8, (int)v13);
  if ( (unsigned __int8)sub_5F55C(a2, a3, 3707583, v13) == 0 )
  {
    sub_12F48(v13);
    (*(void (__fastcall **)(int, int))(*(_DWORD *)v11 + 4))(v11, 1);
    return 0;
  }
  sub_161B0(v11 + 20, (int)v13);
  if ( (unsigned __int8)sub_5F498(a2, a3, -1745176784, &v12) == 0 )
  {
    sub_12F48(v13);
    (*(void (__fastcall **)(int, int))(*(_DWORD *)v11 + 4))(v11, 1);
    return 0;
  }
  *(_BYTE *)(v11 + 64) = v12;
  if ( (unsigned __int8)sub_5F498(a2, a3, -1737881656, &v12) == 0 )
  {
    sub_12F48(v13);
    if ( v11 != 0 )
      (*(void (__fastcall **)(int, int))(*(_DWORD *)v11 + 4))(v11, 1);
    return 0;
  }
  v6 = v12;
  if ( (unsigned __int8)sub_5F498(a2, a3, -52654032, &v12) == 0 )
  {
    sub_12F48(v13);
    if ( v11 != 0 )
      (*(void (__fastcall **)(int, int))(*(_DWORD *)v11 + 4))(v11, 1);
    return 0;
  }
  v7 = v11;
  v8 = v12;
  *(_WORD *)(v11 + 60) = v6;
  *(_WORD *)(v7 + 62) = v8;
  if ( (unsigned __int8)sub_5F498(a2, a3, -1351350021, &v12) == 0 )
  {
    sub_12F48(v13);
    if ( v11 != 0 )
      (*(void (__fastcall **)(int, int))(*(_DWORD *)v11 + 4))(v11, 1);
    return 0;
  }
  *(_DWORD *)(v11 + 52) = sub_19430(v12);
  if ( (unsigned __int8)sub_5F498(a2, a3, 363985063, &v12) == 0 )
  {
    sub_12F48(v13);
    if ( v11 != 0 )
      (*(void (__fastcall **)(int, int))(*(_DWORD *)v11 + 4))(v11, 1);
    return 0;
  }
  *(_DWORD *)(v11 + 32) = sub_3D1B4(v12);
  if ( (unsigned __int8)sub_5F498(a2, a3, -1353917115, &v12) != 0 )
  {
    v9 = 1;
    if ( v12 != 1 )
      v9 = 2;
    *(_DWORD *)(v11 + 56) = v9;
    if ( (unsigned __int8)sub_5F55C(a2, a3, 122350289, v13) != 0 )
    {
      sub_19B0C(v11, 1, v13);
      v10 = v11;
      v11 = 0;
      sub_12EA0((int)v13);
      sub_19604(&v11);
      return v10;
    }
    sub_12F48(v13);
    sub_19604(&v11);
    return 0;
  }
  sub_12F48(v13);
  if ( v11 != 0 )
    (*(void (__fastcall **)(int, int))(*(_DWORD *)v11 + 4))(v11, 1);
  return 0;
}

