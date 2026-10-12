/* ea:   0x000637A0 */
/* name: sub_637A0 */
/* size: 40 */
void *__fastcall sub_637A0(void *a1, char a2)
{
  void *result; // r0

  sub_63374((int)a1);
  result = a1;
  if ( (a2 & 1) != 0 )
  {
    operator delete(a1);
    return a1;
  }
  return result;
}

