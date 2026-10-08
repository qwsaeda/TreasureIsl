/* ea:   0x00013850 */
/* name: sub_13850 */
/* size: 72 */
char *__fastcall sub_13850(_DWORD *a1, int a2)
{
  char *result; // r0

  if ( a2 <= 0 )
    return (char *)TerminateProcess((HANDLE)0x42, 0);
  result = (char *)sub_1343C((int)(a1 + 2), a2);
  *a1 = result;
  a1[1] = result;
  a1[2] = &result[2 * a2];
  return result;
}

