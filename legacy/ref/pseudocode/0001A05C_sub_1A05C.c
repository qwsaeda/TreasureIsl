/* ea:   0x0001A05C */
/* name: sub_1A05C */
/* size: 40 */
void *__fastcall sub_1A05C(void *a1, char a2)
{
  void *result; // r0

  sub_1A084();
  result = a1;
  if ( (a2 & 1) != 0 )
  {
    operator delete(a1);
    return a1;
  }
  return result;
}

