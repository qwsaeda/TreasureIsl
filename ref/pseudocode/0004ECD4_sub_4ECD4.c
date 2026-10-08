/* ea:   0x0004ECD4 */
/* name: sub_4ECD4 */
/* size: 40 */
void *__fastcall sub_4ECD4(void *a1, char a2)
{
  void *result; // r0

  sub_4ECFC(a1);
  result = a1;
  if ( (a2 & 1) != 0 )
  {
    operator delete(a1);
    return a1;
  }
  return result;
}

