/* ea:   0x00066480 */
/* name: sub_66480 */
/* size: 52 */
void *__fastcall sub_66480(int a1, int a2, _BYTE *a3, _BYTE *a4)
{
  void *result; // r0
  void *v7; // r6

  result = sub_65EA4(a1 + 8, a2);
  v7 = result;
  if ( a4 != a3 )
  {
    memmove(result, a3, a4 - a3);
    return v7;
  }
  return result;
}

