/* ea:   0x0004E3CC */
/* name: sub_4E3CC */
/* size: 40 */
void *__fastcall sub_4E3CC(void *a1, char a2)
{
  void *result; // r0

  sub_4E3F4();
  result = a1;
  if ( (a2 & 1) != 0 )
  {
    operator delete(a1);
    return a1;
  }
  return result;
}

