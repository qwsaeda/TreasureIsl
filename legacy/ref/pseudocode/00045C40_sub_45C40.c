/* ea:   0x00045C40 */
/* name: sub_45C40 */
/* size: 40 */
void *__fastcall sub_45C40(void *a1, char a2)
{
  void *result; // r0

  sub_45C68(a1);
  result = a1;
  if ( (a2 & 1) != 0 )
  {
    operator delete(a1);
    return a1;
  }
  return result;
}

