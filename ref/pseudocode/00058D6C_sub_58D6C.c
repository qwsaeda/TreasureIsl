/* ea:   0x00058D6C */
/* name: sub_58D6C */
/* size: 40 */
void *__fastcall sub_58D6C(void *a1, char a2)
{
  void *result; // r0

  sub_58D94(a1);
  result = a1;
  if ( (a2 & 1) != 0 )
  {
    operator delete(a1);
    return a1;
  }
  return result;
}

