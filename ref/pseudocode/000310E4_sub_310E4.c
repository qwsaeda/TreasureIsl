/* ea:   0x000310E4 */
/* name: sub_310E4 */
/* size: 72 */
int __fastcall sub_310E4(_DWORD *a1)
{
  int result; // r0

  result = a1[1];
  if ( result != 0 )
  {
    sub_379F8(a1, *(_DWORD *)(*a1 + 4));
    *(_DWORD *)(*a1 + 8) = *a1;
    *(_DWORD *)(*a1 + 4) = 0;
    result = *a1;
    *(_DWORD *)(*a1 + 12) = *a1;
    a1[1] = 0;
  }
  return result;
}

