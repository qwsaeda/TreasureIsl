/* ea:   0x000430CC */
/* name: sub_430CC */
/* size: 40 */
void *__fastcall sub_430CC(void *a1, char a2)
{
  void *result; // r0

  sub_430F4();
  result = a1;
  if ( (a2 & 1) != 0 )
  {
    operator delete(a1);
    return a1;
  }
  return result;
}

