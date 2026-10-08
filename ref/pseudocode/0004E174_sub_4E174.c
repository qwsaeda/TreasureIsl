/* ea:   0x0004E174 */
/* name: sub_4E174 */
/* size: 40 */
void *__fastcall sub_4E174(void *a1, char a2)
{
  void *result; // r0

  sub_4E19C();
  result = a1;
  if ( (a2 & 1) != 0 )
  {
    operator delete(a1);
    return a1;
  }
  return result;
}

