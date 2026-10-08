/* ea:   0x000635D0 */
/* name: sub_635D0 */
/* size: 24 */
void *__fastcall sub_635D0(void *a1, int a2)
{
  if ( a2 == 0 )
    __debugbreak();
  return memcpy(a1, *(const void **)(a2 + 28), *(_DWORD *)(a2 + 16));
}

