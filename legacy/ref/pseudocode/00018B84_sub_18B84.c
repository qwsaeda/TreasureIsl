/* ea:   0x00018B84 */
/* name: sub_18B84 */
/* size: 40 */
void *__fastcall sub_18B84(void *a1, char a2)
{
  void *result; // r0

  sub_18B10((int)a1);
  result = a1;
  if ( (a2 & 1) != 0 )
  {
    operator delete(a1);
    return a1;
  }
  return result;
}

