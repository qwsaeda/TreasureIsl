/* ea:   0x0004ED78 */
/* name: sub_4ED78 */
/* size: 40 */
void *__fastcall sub_4ED78(void *a1, char a2)
{
  void *result; // r0

  sub_4EDA0();
  result = a1;
  if ( (a2 & 1) != 0 )
  {
    operator delete(a1);
    return a1;
  }
  return result;
}

