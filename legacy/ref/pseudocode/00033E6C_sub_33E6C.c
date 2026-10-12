/* ea:   0x00033E6C */
/* name: sub_33E6C */
/* size: 116 */
int __fastcall sub_33E6C(_DWORD *a1)
{
  if ( a1[4] == a1[5] )
    return 27;
  if ( a1[7] == a1[8] )
    return *(_DWORD *)(a1[4] + 4 * a1[12]);
  return *(_DWORD *)(a1[7] + 4 * a1[13]);
}

