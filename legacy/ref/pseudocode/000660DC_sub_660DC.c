/* ea:   0x000660DC */
/* name: sub_660DC */
/* size: 60 */
void **__fastcall sub_660DC(void **a1, char a2)
{
  void **result; // r0

  *a1 = off_7BD0C;
  sub_12F48(a1 + 1);
  result = a1;
  if ( (a2 & 1) != 0 )
  {
    operator delete(a1);
    return a1;
  }
  return result;
}

