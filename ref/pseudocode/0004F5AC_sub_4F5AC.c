/* ea:   0x0004F5AC */
/* name: sub_4F5AC */
/* size: 40 */
void *__fastcall sub_4F5AC(void *a1, char a2)
{
  void *result; // r0

  sub_4F5D4();
  result = a1;
  if ( (a2 & 1) != 0 )
  {
    operator delete(a1);
    return a1;
  }
  return result;
}

