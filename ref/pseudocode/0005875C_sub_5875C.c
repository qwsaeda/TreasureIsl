/* ea:   0x0005875C */
/* name: sub_5875C */
/* size: 40 */
void *__fastcall sub_5875C(void *a1, char a2)
{
  void *result; // r0

  sub_58784(a1);
  result = a1;
  if ( (a2 & 1) != 0 )
  {
    operator delete(a1);
    return a1;
  }
  return result;
}

