/* ea:   0x00055784 */
/* name: sub_55784 */
/* size: 40 */
void *__fastcall sub_55784(void *a1, char a2)
{
  void *result; // r0

  sub_557AC();
  result = a1;
  if ( (a2 & 1) != 0 )
  {
    operator delete(a1);
    return a1;
  }
  return result;
}

