/* ea:   0x0001102C */
/* name: sub_1102C */
/* size: 40 */
void *__fastcall sub_1102C(void *a1, char a2)
{
  void *result; // r0

  sub_11054();
  result = a1;
  if ( (a2 & 1) != 0 )
  {
    operator delete(a1);
    return a1;
  }
  return result;
}

