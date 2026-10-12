/* ea:   0x00016ED8 */
/* name: sub_16ED8 */
/* size: 72 */
int __fastcall sub_16ED8(_DWORD *a1)
{
  int result; // r0

  result = a1[1];
  if ( result != 0 )
  {
    sub_16F20(a1, *(_DWORD *)(*a1 + 4));
    *(_DWORD *)(*a1 + 8) = *a1;
    *(_DWORD *)(*a1 + 4) = 0;
    result = *a1;
    *(_DWORD *)(*a1 + 12) = *a1;
    a1[1] = 0;
  }
  return result;
}

