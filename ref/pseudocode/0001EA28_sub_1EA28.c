/* ea:   0x0001EA28 */
/* name: sub_1EA28 */
/* size: 52 */
void *__fastcall sub_1EA28(int a1, int a2, _BYTE *a3, _BYTE *a4)
{
  void *result; // r0
  void *v7; // r6

  result = sub_1E598(a1 + 8, a2);
  v7 = result;
  if ( a4 != a3 )
  {
    memmove(result, a3, a4 - a3);
    return v7;
  }
  return result;
}

