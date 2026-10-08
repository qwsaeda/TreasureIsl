/* ea:   0x0006371C */
/* name: sub_6371C */
/* size: 48 */
int __fastcall sub_6371C(int a1)
{
  int result; // r0

  result = *(_DWORD *)(a1 + 44);
  *(_DWORD *)(a1 + 44) = result - 1;
  if ( result == 1 )
    return GXResume();
  return result;
}

