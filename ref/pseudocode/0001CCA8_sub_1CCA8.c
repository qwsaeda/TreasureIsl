/* ea:   0x0001CCA8 */
/* name: sub_1CCA8 */
/* size: 40 */
void *__fastcall sub_1CCA8(void *a1, char a2)
{
  void *result; // r0

  sub_1CCD0();
  result = a1;
  if ( (a2 & 1) != 0 )
  {
    operator delete(a1);
    return a1;
  }
  return result;
}

