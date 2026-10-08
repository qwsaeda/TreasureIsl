/* ea:   0x000115D8 */
/* name: sub_115D8 */
/* size: 92 */
void *__fastcall sub_115D8(size_t a1)
{
  void *result; // r0

  do
  {
    if ( off_81034 == nullptr )
    {
      MessageBoxW(nullptr, aOutOfMemory, nullptr, 0);
      ExitThread(1u);
    }
    off_81034();
    result = malloc(a1);
  }
  while ( result == nullptr );
  return result;
}

