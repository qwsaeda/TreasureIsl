/* ea:   0x0003D18C */
/* name: sub_3D18C */
/* size: 40 */
void **__fastcall sub_3D18C(void **a1, char a2)
{
  void **result; // r0

  sub_16B24(a1);
  result = a1;
  if ( (a2 & 1) != 0 )
  {
    operator delete(a1);
    return a1;
  }
  return result;
}

