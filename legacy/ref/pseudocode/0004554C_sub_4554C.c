/* ea:   0x0004554C */
/* name: sub_4554C */
/* size: 52 */
void *__fastcall sub_4554C(void *a1, char a2)
{
  void *result; // r0

  sub_3D49C((int)a1 + 80);
  sub_45C68(a1);
  result = a1;
  if ( (a2 & 1) != 0 )
  {
    operator delete(a1);
    return a1;
  }
  return result;
}

