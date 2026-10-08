/* ea:   0x0004B43C */
/* name: sub_4B43C */
/* size: 40 */
void *__fastcall sub_4B43C(void *a1, char a2)
{
  void *result; // r0

  sub_4B464(a1);
  result = a1;
  if ( (a2 & 1) != 0 )
  {
    operator delete(a1);
    return a1;
  }
  return result;
}

