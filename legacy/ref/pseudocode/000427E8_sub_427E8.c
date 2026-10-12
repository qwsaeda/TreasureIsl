/* ea:   0x000427E8 */
/* name: sub_427E8 */
/* size: 264 */
int __fastcall sub_427E8(int *a1)
{
  int v2; // r1
  unsigned __int16 *v3; // r8
  int v4; // r7
  int v5; // r6
  int v6; // r2
  int v7; // r1
  unsigned int v8; // r5
  int v9; // r4
  int v10; // r0
  int v11; // t1
  int v12; // r3
  int v13; // r1
  int result; // r0

  v2 = a1[12];
  if ( v2 != 0 )
    (*(void (__fastcall **)(int))(*(_DWORD *)v2 + 12))(a1[12]);
  v3 = (unsigned __int16 *)a1[13];
  v4 = a1[14];
  v5 = 5;
  v6 = *v3;
  v7 = v6;
  if ( *v3 != 0 )
  {
    do
    {
      if ( v7 == 65283 )
      {
        v4 += 14;
        v5 = 5;
      }
      else if ( v4 > 6 && v4 < 202 )
      {
        v8 = a1[17];
        v9 = a1[2];
        v10 = sub_1B06C(v8, v6);
        sub_1B150(v8, v9, v5, v4, 12451, v10);
        v5 += 14;
      }
      v11 = v3[1];
      ++v3;
      v6 = v11;
      v7 = (unsigned __int16)v11;
    }
    while ( (_WORD)v11 != 0 );
  }
  if ( v4 <= 0 )
    return sub_3DC04(a1[1], 6);
  v12 = a1[15];
  v13 = v12 / 2;
  if ( v12 % 2 != 0 )
    v13 = a1[14] - 1;
  result = v12 + 1;
  if ( v12 % 2 != 0 )
    a1[14] = v13;
  a1[15] = result;
  return result;
}

