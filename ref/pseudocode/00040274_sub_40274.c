/* ea:   0x00040274 */
/* name: sub_40274 */
/* size: 40 */
void *__fastcall sub_40274(void *a1, char a2)
{
  void *result; // r0

  sub_4029C();
  result = a1;
  if ( (a2 & 1) != 0 )
  {
    operator delete(a1);
    return a1;
  }
  return result;
}

