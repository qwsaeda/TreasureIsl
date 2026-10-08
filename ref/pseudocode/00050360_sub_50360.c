/* ea:   0x00050360 */
/* name: sub_50360 */
/* size: 40 */
void *__fastcall sub_50360(void *a1, char a2)
{
  void *result; // r0

  sub_50388(a1);
  result = a1;
  if ( (a2 & 1) != 0 )
  {
    operator delete(a1);
    return a1;
  }
  return result;
}

