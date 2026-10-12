/* ea:   0x0004E638 */
/* name: sub_4E638 */
/* size: 40 */
void *__fastcall sub_4E638(void *a1, char a2)
{
  void *result; // r0

  sub_4E660();
  result = a1;
  if ( (a2 & 1) != 0 )
  {
    operator delete(a1);
    return a1;
  }
  return result;
}

