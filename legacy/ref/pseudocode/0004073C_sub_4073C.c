/* ea:   0x0004073C */
/* name: sub_4073C */
/* size: 124 */
int __fastcall sub_4073C(_DWORD *a1)
{
  int v2; // r1
  int result; // r0

  v2 = a1[23];
  if ( v2 != 0 )
    return (*(int (__fastcall **)(_DWORD))(*(_DWORD *)v2 + 12))(a1[23]);
  result = a1[12];
  if ( a1[12] != 0 )
  {
    (*(void (__fastcall **)(_DWORD))(*(_DWORD *)a1[12] + 12))(a1[12]);
    sub_44648(a1[12], *(_DWORD *)(a1[16] + 4 * a1[22]) + 6);
    return sub_40C68(a1);
  }
  return result;
}

