/* ea:   0x0001F04C */
/* name: sub_1F04C */
/* size: 40 */
void *__fastcall sub_1F04C(void *a1, char a2)
{
  void *result; // r0

  sub_1F0C8();
  result = a1;
  if ( (a2 & 1) != 0 )
  {
    operator delete(a1);
    return a1;
  }
  return result;
}

