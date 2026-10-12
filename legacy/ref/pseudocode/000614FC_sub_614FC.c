/* ea:   0x000614FC */
/* name: sub_614FC */
/* size: 40 */
HANDLE *__fastcall sub_614FC(HANDLE *a1, char a2)
{
  HANDLE *result; // r0

  sub_6118C(a1);
  result = a1;
  if ( (a2 & 1) != 0 )
  {
    operator delete(a1);
    return a1;
  }
  return result;
}

