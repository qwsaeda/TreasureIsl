/* ea:   0x00066914 */
/* name: sub_66914 */
/* size: 40 */
void *__fastcall sub_66914(void *a1, char a2)
{
  void *result; // r0

  sub_651B8((int)a1);
  result = a1;
  if ( (a2 & 1) != 0 )
  {
    operator delete(a1);
    return a1;
  }
  return result;
}

