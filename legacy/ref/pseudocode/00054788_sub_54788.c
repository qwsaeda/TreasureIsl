/* ea:   0x00054788 */
/* name: sub_54788 */
/* size: 40 */
void *__fastcall sub_54788(void *a1, char a2)
{
  void *result; // r0

  sub_547B0();
  result = a1;
  if ( (a2 & 1) != 0 )
  {
    operator delete(a1);
    return a1;
  }
  return result;
}

