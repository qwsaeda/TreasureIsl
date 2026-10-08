/* ea:   0x000466F0 */
/* name: sub_466F0 */
/* size: 40 */
void *__fastcall sub_466F0(void *a1, char a2)
{
  void *result; // r0

  sub_46718();
  result = a1;
  if ( (a2 & 1) != 0 )
  {
    operator delete(a1);
    return a1;
  }
  return result;
}

