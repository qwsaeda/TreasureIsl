/* ea:   0x00046314 */
/* name: sub_46314 */
/* size: 40 */
void *__fastcall sub_46314(void *a1, char a2)
{
  void *result; // r0

  sub_4633C();
  result = a1;
  if ( (a2 & 1) != 0 )
  {
    operator delete(a1);
    return a1;
  }
  return result;
}

