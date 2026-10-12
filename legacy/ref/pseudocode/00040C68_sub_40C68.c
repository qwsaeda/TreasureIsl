/* ea:   0x00040C68 */
/* name: sub_40C68 */
/* size: 216 */
int __fastcall sub_40C68(_DWORD *a1)
{
  int v2; // r9
  int v3; // r7
  int result; // r0
  wchar_t *v5; // r3
  wchar_t *v6; // r6
  unsigned int v7; // r5
  int v8; // r4
  int v9; // r0
  wchar_t v10[50]; // [sp+8h] [bp-64h] BYREF

  v2 = 31;
  v3 = 153;
  memset(v10, 0, 0x40u);
  swprintf(v10, off_6EF34[a1[22]]);
  result = v10[0] << 16;
  if ( v10[0] != 0 )
  {
    v5 = v10;
    v6 = v10;
    do
    {
      if ( *v5 == 65283 )
      {
        v2 = 31;
        v3 += 13;
      }
      else
      {
        v7 = a1[25];
        v8 = a1[2];
        v9 = sub_1B06C(v7);
        sub_1B150(v7, v8, v2, v3, 0xFFFF, v9);
        v2 += 12;
      }
      result = *++v6;
      v5 = v6;
    }
    while ( *v6 != 0 );
  }
  return result;
}

