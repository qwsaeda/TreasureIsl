/* ea:   0x0003E614 */
/* name: sub_3E614 */
/* size: 72 */
int __fastcall sub_3E614(_DWORD *a1)
{
  int result; // r0

  result = a1[1];
  if ( result != 0 )
  {
    sub_3E65C(a1, *(_DWORD *)(*a1 + 4));
    *(_DWORD *)(*a1 + 8) = *a1;
    *(_DWORD *)(*a1 + 4) = 0;
    result = *a1;
    *(_DWORD *)(*a1 + 12) = *a1;
    a1[1] = 0;
  }
  return result;
}

