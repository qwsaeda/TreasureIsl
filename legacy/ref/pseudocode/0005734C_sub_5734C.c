/* ea:   0x0005734C */
/* name: sub_5734C */
/* size: 40 */
void *__fastcall sub_5734C(void *a1, char a2)
{
  void *result; // r0

  sub_57374();
  result = a1;
  if ( (a2 & 1) != 0 )
  {
    operator delete(a1);
    return a1;
  }
  return result;
}

