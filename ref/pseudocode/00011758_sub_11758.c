/* ea:   0x00011758 */
/* name: sub_11758 */
/* size: 40 */
void *__fastcall sub_11758(void *a1, char a2)
{
  void *result; // r0

  sub_1179C();
  result = a1;
  if ( (a2 & 1) != 0 )
  {
    operator delete(a1);
    return a1;
  }
  return result;
}

