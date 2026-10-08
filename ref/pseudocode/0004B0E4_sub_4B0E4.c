/* ea:   0x0004B0E4 */
/* name: sub_4B0E4 */
/* size: 40 */
void *__fastcall sub_4B0E4(void *a1, char a2)
{
  void *result; // r0

  sub_4B10C();
  result = a1;
  if ( (a2 & 1) != 0 )
  {
    operator delete(a1);
    return a1;
  }
  return result;
}

