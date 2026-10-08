/* ea:   0x00054C98 */
/* name: sub_54C98 */
/* size: 124 */
BOOL __fastcall sub_54C98(_DWORD *a1)
{
  int v2; // r1
  int v3; // r0
  int i; // r3

  v2 = a1[257];
  v3 = a1[258];
  for ( i = 0; v2 != v3; ++i )
    v2 += 8;
  if ( a1[278] != 0 )
    ++i;
  if ( a1[279] != 0 )
    ++i;
  if ( a1[280] != 0 )
    ++i;
  if ( a1[281] != 0 )
    ++i;
  if ( a1[282] != 0 )
    ++i;
  return i < 6;
}

