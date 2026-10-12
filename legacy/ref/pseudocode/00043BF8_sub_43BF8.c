/* ea:   0x00043BF8 */
/* name: sub_43BF8 */
/* size: 40 */
void *__fastcall sub_43BF8(void *a1, char a2)
{
  void *result; // r0

  sub_43C20(a1);
  result = a1;
  if ( (a2 & 1) != 0 )
  {
    operator delete(a1);
    return a1;
  }
  return result;
}

