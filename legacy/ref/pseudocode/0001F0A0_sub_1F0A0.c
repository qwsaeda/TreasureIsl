/* ea:   0x0001F0A0 */
/* name: sub_1F0A0 */
/* size: 40 */
void *__fastcall sub_1F0A0(void *a1, char a2)
{
  void *result; // r0

  sub_3CE10();
  result = a1;
  if ( (a2 & 1) != 0 )
  {
    operator delete(a1);
    return a1;
  }
  return result;
}

