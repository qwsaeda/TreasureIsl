/* ea:   0x0004AF80 */
/* name: sub_4AF80 */
/* size: 76 */
int __fastcall sub_4AF80(int a1, int a2, int a3, int *a4)
{
  int v4; // r2
  int result; // r0

  v4 = *a4;
  result = a2 == (_DWORD)a4;
  if ( (int *)a2 != a4 )
  {
    result = a2 == v4;
    if ( a2 != v4 )
      return sub_39D90(a2, (int)a4, v4);
  }
  return result;
}

