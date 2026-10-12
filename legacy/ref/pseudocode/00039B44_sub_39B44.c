/* ea:   0x00039B44 */
/* name: sub_39B44 */
/* size: 76 */
int __fastcall sub_39B44(int a1, _DWORD *a2, int a3, _DWORD *a4)
{
  int result; // r0

  result = a2 == a4;
  if ( a2 != a4 )
  {
    result = a2 == (_DWORD *)*a4;
    if ( a2 != (_DWORD *)*a4 )
      return sub_39D90(a2, a4);
  }
  return result;
}

