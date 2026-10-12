/* ea:   0x000459E8 */
/* name: sub_459E8 */
/* size: 40 */
void *__fastcall sub_459E8(void *a1, char a2)
{
  void *result; // r0

  sub_45A10();
  result = a1;
  if ( (a2 & 1) != 0 )
  {
    operator delete(a1);
    return a1;
  }
  return result;
}

