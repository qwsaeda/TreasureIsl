/* ea:   0x00065204 */
/* name: sub_65204 */
/* size: 296 */
BOOL __fastcall sub_65204(_DWORD *a1)
{
  _DWORD *v2; // r7
  unsigned int i; // r4
  unsigned int v4; // r2
  _BYTE *v6; // [sp+0h] [bp-24h] BYREF
  int v7; // [sp+4h] [bp-20h] BYREF

  v2 = a1 + 5;
  if ( a1[5] != a1[6] || a1[8] != a1[9] )
    sub_6532C(a1);
  if ( a1[4] == 0 )
    return true;
  sub_661C8(v2, a1[4]);
  sub_66258(a1 + 8, a1[4]);
  for ( i = 0; i < a1[4]; ++i )
  {
    v6 = sub_64EB8();
    if ( v6 == nullptr )
      break;
    sub_65AB4(v2, &v6);
    v7 = sub_65124();
    if ( v7 == 0 )
      break;
    sub_65B14(a1 + 8, &v7);
  }
  v4 = a1[4];
  return v4 <= (v2[1] - *v2) >> 2 && v4 <= (a1[9] - a1[8]) >> 2;
}

