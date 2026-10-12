/* ea:   0x00043434 */
/* name: sub_43434 */
/* size: 40 */
void *__fastcall sub_43434(void *a1, char a2)
{
  void *result; // r0

  sub_4345C();
  result = a1;
  if ( (a2 & 1) != 0 )
  {
    operator delete(a1);
    return a1;
  }
  return result;
}

