/* ea:   0x0003D034 */
/* name: sub_3D034 */
/* size: 344 */
int __fastcall sub_3D034(int a1, int a2, int a3)
{
  int v6; // r4
  int v7; // [sp+4h] [bp-24h] BYREF
  unsigned __int8 v8[8]; // [sp+8h] [bp-20h] BYREF
  void *v9[6]; // [sp+10h] [bp-18h] BYREF

  if ( (unsigned __int8)sub_5F498(a2, a3, 3565, v8) == 0 )
    return 0;
  v7 = sub_3A6D8(v8[0]);
  if ( v7 == 0 )
  {
    sub_3D258(&v7);
    return 0;
  }
  sub_1333C(v9);
  if ( (unsigned __int8)sub_5F55C(a2, a3, 4062401, v9) != 0 )
  {
    if ( v9 != (void **)(v7 + 8) )
      sub_17890(v7 + 8, (__int16 *)v9[0], (__int16 *)v9[1]);
    if ( (unsigned __int8)sub_5F55C(a2, a3, 122350289, v9) != 0 )
    {
      sub_161B0(v7 + 36, (int)v9);
      if ( (unsigned __int8)sub_5F498(a2, a3, 363985063, v8) != 0 )
      {
        *(_DWORD *)(v7 + 32) = sub_3D1B4(*(_DWORD *)v8);
        v6 = v7;
        v7 = 0;
        sub_12EA0((int)v9);
        sub_3D258(&v7);
        return v6;
      }
    }
  }
  sub_132A0(v9);
  sub_3D18C(v7, 1);
  return 0;
}

