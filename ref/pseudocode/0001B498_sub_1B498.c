/* ea:   0x0001B498 */
/* name: sub_1B498 */
/* size: 40 */
void *__fastcall sub_1B498(void *a1, char a2)
{
  void *result; // r0

  sub_1B4C0();
  result = a1;
  if ( (a2 & 1) != 0 )
  {
    operator delete(a1);
    return a1;
  }
  return result;
}

